use async_trait::async_trait;
use sqlx::{PgPool, query};
use tracing::instrument;

use crate::domain::{Email, HashedPassword, User, UserStore, UserStoreError};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserStore for PostgresUserStore {
    #[instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            user.email.as_ref(),
            user.password.as_ref(),
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                UserStoreError::UserAlreadyExists
            }
            _ => UserStoreError::UnexpectedError,
        })?;

        Ok(())
    }

    #[instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let row = query!(
            "SELECT email, password_hash, requires_2fa FROM users WHERE email = $1",
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        match row {
            Some(data) => {
                let email = data
                    .email
                    .parse()
                    .map_err(|_| UserStoreError::UnexpectedError)?;
                let password = HashedPassword::parse_password_hash(data.password_hash)
                    .map_err(|_| UserStoreError::UnexpectedError)?;

                Ok(User {
                    email,
                    password,
                    requires_2fa: data.requires_2fa,
                })
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }

    #[instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(&self, email: &Email, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        user.password
            .verify_raw_password(password)
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)
    }
}
