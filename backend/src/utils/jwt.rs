use crate::models::Claims;
use crate::config::Config;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use uuid::Uuid;
use actix_web::{Error, error::ErrorUnauthorized};

pub fn generate_token(user_id: Uuid, username: &str, config: &Config) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + Duration::hours(config.jwt_expiry_hours);
    
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
}

pub fn validate_token(token: &str, config: &Config) -> Result<Claims, Error> {
    let token = token.strip_prefix("Bearer ").unwrap_or(token);
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ErrorUnauthorized("Invalid or expired token"))
}
