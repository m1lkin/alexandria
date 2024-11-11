use std::fmt::{Display};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AppError {
    BadRequest,
    Conflict,
    InternalServerError,
    NotFound,
    NotAuthorized,
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Conflict => write!(f, "conflict"),
            AppError::NotFound => write!(f, "not found"),
            AppError::BadRequest => write!(f, "bad request"),
            AppError::NotAuthorized => write!(f, "not authorized"),
            AppError::InternalServerError => write!(f, "internal server error")
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest => {StatusCode::BAD_REQUEST.into_response()}
            AppError::Conflict => {StatusCode::CONFLICT.into_response()}
            AppError::InternalServerError => {StatusCode::INTERNAL_SERVER_ERROR.into_response()}
            AppError::NotFound => {StatusCode::NOT_FOUND.into_response()}
            AppError::NotAuthorized => {StatusCode::UNAUTHORIZED.into_response()}
        }
    }
}