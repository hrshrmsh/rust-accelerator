use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ErrorResponse,
    domain::{TokenStoreError, UserStoreError},
    utils::auth::GenerateTokenError,
};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AuthAPIError {
    #[error("User already exists!")]
    UserAlreadyExists,
    #[error("Invalid credentials!")]
    InvalidCredentials,
    #[error("Unexpected error!")]
    UnexpectedError,
    #[error("Authentication failed!")]
    AuthenticationError,
    #[error("Missing Token!")]
    MissingToken,
    #[error("JWT is not valid!")]
    InvalidToken,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::UserAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidCredentials | Self::MissingToken => StatusCode::BAD_REQUEST,
            Self::AuthenticationError | Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}

impl From<UserStoreError> for AuthAPIError {
    fn from(value: UserStoreError) -> Self {
        match value {
            UserStoreError::UserAlreadyExists => Self::UserAlreadyExists,
            UserStoreError::InvalidCredentials | UserStoreError::UserNotFound => {
                Self::AuthenticationError
            }
            _ => Self::UnexpectedError,
        }
    }
}

impl From<GenerateTokenError> for AuthAPIError {
    fn from(value: GenerateTokenError) -> Self {
        match value {
            GenerateTokenError::TokenError(_) => Self::InvalidToken,
            GenerateTokenError::UnexpectedError => Self::UnexpectedError,
            GenerateTokenError::BannedToken => Self::InvalidToken,
        }
    }
}

impl From<TokenStoreError> for AuthAPIError {
    fn from(value: TokenStoreError) -> Self {
        match value {
            TokenStoreError::MissingToken => Self::MissingToken,
            TokenStoreError::UnexpectedError => Self::UnexpectedError,
        }
    }
}
