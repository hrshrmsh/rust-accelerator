use crate::domain::{LoginAttemptId, TwoFACode};

use async_trait::async_trait;

use crate::domain::{Email, User};

#[async_trait]
pub trait UserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, raw_password: &str) -> Result<(), UserStoreError>;
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

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Debug, PartialEq)]
pub enum TokenStoreError {
    MissingToken,
    UnexpectedError,
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}
