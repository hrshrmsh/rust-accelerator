use std::sync::Arc;

use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    domain::{BannedTokenStore, Email},
    utils::constants::{JWT_COOKIE_NAME, JWT_SECRET},
};

pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

fn create_auth_cookie(token: String) -> Cookie<'static> {
    Cookie::build((JWT_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build()
}

#[derive(Debug, Error)]
pub enum GenerateTokenError {
    #[error("{0}")]
    TokenError(#[from] jsonwebtoken::errors::Error),
    #[error("banned jwt token")]
    BannedToken,
    #[error("unexpected error")]
    UnexpectedError,
}

// 10 min
pub const TOKEN_TTL_SECONDS: i64 = 600;

fn generate_auth_token(email: &Email) -> Result<String, GenerateTokenError> {
    let delta =
        Duration::try_seconds(TOKEN_TTL_SECONDS).ok_or(GenerateTokenError::UnexpectedError)?;

    let expiration: usize = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp()
        .try_into()
        .map_err(|_| GenerateTokenError::UnexpectedError)?;

    let claims = Claims {
        subject: email.as_ref().to_string(),
        expirary: expiration,
    };

    create_token(&claims)
}

pub async fn validate_token(
    banned_token_store: Arc<dyn BannedTokenStore + Send + Sync>,
    token: &str,
) -> Result<Claims, GenerateTokenError> {
    if banned_token_store
        .check_token(token)
        .await
        .map_err(|_| GenerateTokenError::UnexpectedError)?
    {
        Err(GenerateTokenError::BannedToken)
    } else {
        Ok(decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)?)
    }
}

fn create_token(claims: &Claims) -> Result<String, GenerateTokenError> {
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(rename = "sub")]
    pub subject: String,
    #[serde(rename = "exp")]
    pub expirary: usize,
}

#[cfg(test)]
mod tests {
    use crate::services::HashSetTokenStore;

    use super::*;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email: Email = "test@example.com".parse().unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();

        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_string();
        let cookie = create_auth_cookie(token.clone());

        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email: Email = "test@example.com".parse().unwrap();
        let result = generate_auth_token(&email).unwrap();

        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email: Email = "test@example.com".parse().unwrap();
        let token = generate_auth_token(&email).unwrap();
        let banned_token_store = Arc::new(HashSetTokenStore::default());
        let result = validate_token(banned_token_store.clone(), &token)
            .await
            .unwrap();
        assert_eq!(result.subject, "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.expirary > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_string();
        let banned_token_store = Arc::new(HashSetTokenStore::default());
        let result = validate_token(banned_token_store, &token).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_token_with_banned_token() {
        let email: Email = "test@example.com".parse().unwrap();
        let token = generate_auth_token(&email).unwrap();
        let banned_token_store = Arc::new(HashSetTokenStore::default());
        banned_token_store.add_token(&token).await.unwrap();

        let result = validate_token(banned_token_store, &token).await;

        assert!(matches!(result, Err(GenerateTokenError::BannedToken)));
    }
}
