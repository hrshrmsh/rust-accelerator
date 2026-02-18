use async_trait::async_trait;

use crate::domain::{Email, Password, User};

#[async_trait]
pub trait UserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
    -> Result<(), UserStoreError>;
}

#[async_trait]
pub trait BannedTokenStore {
    async fn add_token(&self, token: &str) -> Result<(), TokenStoreError>;
    async fn check_token(&self, token: &str) -> Result<bool, TokenStoreError>;
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
