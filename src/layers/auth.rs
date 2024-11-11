use axum::extract::Request;
use axum::http::header::{AUTHORIZATION};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use chrono::Utc;
use crate::error::AppError;
use crate::hash::validate_token;

pub async fn auth(
    headers: HeaderMap,
    mut request: Request,
    next: Next
) -> Result<Response, AppError> {
    let header = headers.get(AUTHORIZATION)
        .ok_or(AppError::BadRequest)?
        .to_str()
        .map_err(|_| AppError::BadRequest)?;

    if !header.starts_with("Bearer ") {
        return Err(AppError::BadRequest);
    }

    let claims = validate_token(header[7..].to_string())?;

    if claims.exp < Utc::now().timestamp() {
        return Err(AppError::NotAuthorized)
    }

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
