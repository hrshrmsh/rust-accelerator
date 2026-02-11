use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{domain::UserStoreError, ErrorResponse};

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
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::UserAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidCredentials => StatusCode::BAD_REQUEST,
            Self::AuthenticationError => StatusCode::UNAUTHORIZED,
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
            _ => AuthAPIError::UnexpectedError,
        }
    }
}
