use std::sync::Arc;
use axum::extract::{State};
use axum::{debug_handler, Extension, Json};
use axum_extra::extract::Query;
use bson::doc;
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde::Deserialize;
use crate::AppState;
use crate::db::{create_record, get_record, update_record};
use crate::error::AppError;
use crate::structures::{Claims, CreateResource, RatedPost, Rating, Resource, SendResource, User};

#[derive(Deserialize)]
pub struct GetParams {
    posts: Vec<i64>,
    keywords: Vec<String>,
}

pub async fn get_posts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(mut params): Query<GetParams>
) -> Result<Json<Vec<SendResource>>, AppError> {
    let posts: Collection<Resource> = state.client.database("alexandria").collection("posts");
    let user: User = get_record(&claims.sub, &state.client.database("alexandria").collection("users")).await?;
    
    let mut result = vec![];
    if params.posts.pop() == Some(0) {
        let mut cursor = posts.find(doc! {})
            .sort(doc! {"upload_time": -1}).await
            .map_err(|_| AppError::InternalServerError)?;
        for _ in 1..=10 {
            if let Ok(Some(v)) = cursor.try_next().await {
                result.push(v);
            }
        }
    } else {
        for post_id in params.posts {
            result.push(get_record(&post_id, &posts).await?);
        }
    }
    
    let mut posts = vec![];
    
    for post in result.into_iter() {
        if let Some(pt) = user.rated.iter().find(|x| x.post == post.id) {
            posts.push(post.into_send_resource(pt.clone().rating));
        } else {
            posts.push(post.into_send_resource(Rating::None));
        }
    }
    
    Ok(Json(posts))
}

#[debug_handler]
pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateResource>,
) -> Result<Json<i64>, AppError> {
    let db = state.client.database("alexandria");
    let posts: Collection<Resource> = db.collection("posts");
    let user: User = get_record(&claims.sub, &db.collection("users")).await?;

    match create_record(&payload.into_resource(user.id, user.username, &state.id_gen).await?, &posts).await {
        Ok(v) => Ok(Json(v.inserted_id.as_i64().unwrap())),
        Err(e) => Err(e)
    }
}

pub async fn rate_post(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Claims>,
    Json(payload): Json<RatedPost>,
) -> Result<Json<Resource>, AppError> {
    let db = state.client.database("alexandria");
    let posts: Collection<Resource> = db.collection("posts");
    let mut post = get_record(&payload.post, &posts).await?;
    let mut user: User = get_record(&user.sub, &db.collection("users")).await?;
    post.rating += user.add_rated(payload);
    update_record(&post.id, &post, &posts).await?;
    update_record(&user.id, &user, &db.collection("users")).await?;

    Ok(Json(post))
}