use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use color_eyre::Report;
use std::fmt::Write;
use thiserror::Error;

use crate::{
    ErrorResponse,
    domain::{TokenStoreError, TwoFACodeStoreError, UserStoreError},
    utils::auth::GenerateTokenError,
};

#[derive(Error, Debug)]
pub enum AuthAPIError {
    #[error("User already exists!")]
    UserAlreadyExists,
    #[error("Invalid credentials!")]
    InvalidCredentials,
    #[error("Authentication failed!")]
    AuthenticationError,
    #[error("Missing Token!")]
    MissingToken,
    #[error("JWT is not valid!")]
    InvalidToken,
    #[error("Invalid 2FA code!")]
    Invalid2FACode,
    #[error("Invalid login attempt id!")]
    InvalidLoginAttemptId,
    #[error("Unexpected error!")]
    UnexpectedError(#[source] Report),
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        log_error_chain(&self);

        let status = match self {
            Self::UserAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidCredentials
            | Self::MissingToken
            | Self::Invalid2FACode
            | Self::InvalidLoginAttemptId => StatusCode::BAD_REQUEST,
            Self::AuthenticationError | Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}

fn log_error_chain(e: &(dyn std::error::Error + 'static)) {
    let separator =
        "\n-----------------------------------------------------------------------------------\n";
    let mut report = format!("{}{:?}\n", separator, e);
    let mut current = e.source();
    while let Some(cause) = current {
        let str = format!("Caused by:\n\n{:?}", cause);
        // str writes only fail due to OOM - out of scope to handle
        write!(report, "\n{}", str).unwrap();
        current = cause.source();
    }
    write!(report, "\n{}", separator).unwrap();
    tracing::error!("{}", report);
}

impl From<UserStoreError> for AuthAPIError {
    fn from(value: UserStoreError) -> Self {
        match value {
            UserStoreError::UserAlreadyExists => Self::UserAlreadyExists,
            UserStoreError::InvalidCredentials | UserStoreError::UserNotFound => {
                Self::AuthenticationError
            }
            UserStoreError::UnexpectedError(e) => Self::UnexpectedError(e),
        }
    }
}

impl From<GenerateTokenError> for AuthAPIError {
    fn from(value: GenerateTokenError) -> Self {
        match value {
            GenerateTokenError::UnexpectedError(e) => Self::UnexpectedError(e),
            GenerateTokenError::TokenError(_) | GenerateTokenError::BannedToken => {
                Self::InvalidToken
            }
        }
    }
}

impl From<TokenStoreError> for AuthAPIError {
    fn from(value: TokenStoreError) -> Self {
        match value {
            TokenStoreError::MissingToken => Self::MissingToken,
            TokenStoreError::UnexpectedError(e) => Self::UnexpectedError(e),
        }
    }
}

impl From<TwoFACodeStoreError> for AuthAPIError {
    fn from(value: TwoFACodeStoreError) -> Self {
        match value {
            TwoFACodeStoreError::LoginAttemptIdNotFound => Self::InvalidLoginAttemptId,
            TwoFACodeStoreError::UnexpectedError(e) => Self::UnexpectedError(e),
        }
    }
}

impl From<Report> for AuthAPIError {
    fn from(value: Report) -> Self {
        Self::UnexpectedError(value.wrap_err("Unexpected error!"))
    }
}
