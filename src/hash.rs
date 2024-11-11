use std::str::FromStr;
use argon2_kdf::{Hash, Hasher};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use crate::error::AppError;
use crate::structures::Claims;

pub fn generate_token(id: String) -> Result<String, AppError> {
    let info = Claims {sub: id, exp: Utc::now().timestamp() + 60 * 60 * 24 * 7};
    match encode(
        &Header::default(),
        &info,
        &EncodingKey::from_secret(std::env::var("SECRET").map_err(|e| AppError::InternalServerError)?.as_bytes())
    ) {
        Ok(token) => Ok(token),
        Err(_) => Err(AppError::InternalServerError)
    }
}

pub fn validate_token(token: String) -> Result<Claims, AppError> {
    match decode(&token, &DecodingKey::from_secret(std::env::var("SECRET").map_err(|_| AppError::InternalServerError)?.as_bytes()), &Validation::default()) {
        Ok(data) => Ok(data.claims),
        Err(_) => Err(AppError::InternalServerError)
    }
}

pub fn hash_password(password: String) -> Result<String, AppError> {
    match Hasher::default().hash(password.as_bytes()) {
        Ok(hash) => Ok(hash.to_string()),
        Err(_) => Err(AppError::InternalServerError)
    }
}

pub fn verify_password(password: String, hash: String) -> bool {
    Hash::from_str(hash.as_str()).unwrap().verify(password.as_bytes())
}