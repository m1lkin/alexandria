use std::sync::Arc;
use axum::extract::State;
use axum::http::{StatusCode};
use axum::{debug_handler, Extension, Json};
use bson::doc;
use futures_util::TryStreamExt;
use mongodb::Collection;
use crate::AppState;
use crate::db::{create_record, get_record, update_record};
use crate::error::AppError;
use crate::structures::{Claims, CreateResource, File, RatedPost, Rating, Resource, User};

pub async fn get_posts(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Vec<bson::Uuid>>,
) -> Result<Json<Vec<Resource>>, AppError> {
    let posts: Collection<Resource> = state.client.database("alexandria").collection("posts");
    let mut result = vec![];
    if payload.is_empty() {
        let mut cursor = posts.find(doc! {})
            .sort(doc! {"upload_time": -1}).await
            .map_err(|_| AppError::InternalServerError)?;
        for _ in 1..=10 {
            if let Ok(Some(v)) = cursor.try_next().await {
                result.push(v);
            }
        }
        return Ok(Json(result));
    }

    for post_id in payload {
        result.push(get_record(&post_id, &posts).await?);
    }

    Ok(Json(result))
}

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateResource>,
) -> Result<StatusCode, AppError> {
    let db = state.client.database("alexandria");
    let files: Collection<File> = db.collection("files");
    let posts: Collection<Resource> = db.collection("posts");
    let user: User = get_record(&claims.sub, &db.collection("users")).await?;

    match create_record(&payload.into_resource(user.id, user.username, &state.id_gen, &files).await?, &posts).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => Err(e)
    }
}

#[debug_handler]
pub async fn rate_post(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Claims>,
    Json(payload): Json<RatedPost>,
) -> Result<Json<Resource>, AppError> {
    let db = state.client.database("alexandria");
    let posts: Collection<Resource> = db.collection("posts");
    let mut post = get_record(&payload.post, &posts).await?;

    match payload.rating {
        Rating::Up => post.rating += 1,
        Rating::Down => post.rating -= 1,
    }

    let mut user: User = get_record(&user.sub, &db.collection("users")).await?;
    user.add_rated(payload);
    update_record(&post.id, &post, &posts).await?;
    update_record(&user.id, &user, &db.collection("users")).await?;

    Ok(Json(post))
}