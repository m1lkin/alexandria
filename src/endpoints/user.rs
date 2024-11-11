use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use mongodb::Collection;
use crate::AppState;
use crate::db::{create_record, get_record};
use crate::error::AppError;
use crate::hash::{generate_token, hash_password, verify_password};
use crate::structures::{Claims, User};

pub async fn register(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>
) -> Result<StatusCode, AppError> {
    let database = state.client.database("alexandria");
    let users: Collection<User> = database.collection("users");

    let password = params.get("password").ok_or(AppError::BadRequest)?;
    let username = params.get("username").ok_or(AppError::BadRequest)?;
    let email = params.get("email").ok_or(AppError::BadRequest)?;

    if get_record(email, &users).await.is_ok() {
        return Err(AppError::Conflict);
    }

    let result = create_record(&User::new(
        email.to_string(),
        username.to_string(),
        hash_password(password.to_owned())?
    ), &users).await;

    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => Err(e),
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>
) -> Result<Json<(User, String)>, AppError>{
    let database = state.client.database("alexandria");
    let users: Collection<User> = database.collection("users");

    let user_id = params.get("id").ok_or(AppError::BadRequest)?;
    let password = params.get("password").ok_or(AppError::BadRequest)?;

    match get_record(user_id, &users).await {
        Ok(user) => {
            if verify_password(password.to_owned(), user.password_hash.clone()) {
                return Ok(Json((user, generate_token(user_id.to_string())?)))
            }
            Err(AppError::NotAuthorized)
        },
        Err(e) => Err(e),
    }
}

pub async fn update_token(
    Extension(claims): Extension<Claims>
) -> Result<String, AppError> {
    generate_token(claims.sub)
}