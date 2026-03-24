use crate::domain::{LoginAttemptId, TwoFACode};

use async_trait::async_trait;
use color_eyre::Report;
use secrecy::SecretString;
use thiserror::Error;

use crate::domain::{Email, User};

#[async_trait]
pub trait UserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(
        &self,
        email: &Email,
        raw_password: &SecretString,
    ) -> Result<(), UserStoreError>;
}

#[async_trait]
pub trait BannedTokenStore {
    async fn add_token(&self, token: &str) -> Result<(), TokenStoreError>;
    async fn check_token(&self, token: &str) -> Result<bool, TokenStoreError>;
}

#[async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Error, Debug)]
pub enum UserStoreError {
    #[error("User already exists!")]
    UserAlreadyExists,
    #[error("User not found!")]
    UserNotFound,
    #[error("Invalid credentials!")]
    InvalidCredentials,
    #[error("Unexpected error!")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Debug, Error)]
pub enum TokenStoreError {
    #[error("Missing token!")]
    MissingToken,
    #[error("Unexpected error!")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for TokenStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::MissingToken, Self::MissingToken)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Error, Debug)]
pub enum TwoFACodeStoreError {
    #[error("Login attempt not found!")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error!")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}
