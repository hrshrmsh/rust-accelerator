use std::{num::TryFromIntError, sync::Arc};

use async_trait::async_trait;
use redis::{AsyncTypedCommands, SetExpiry, SetOptions, aio::MultiplexedConnection};
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{
    domain::{BannedTokenStore, TokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

#[derive(Clone)]
pub struct RedisBannedTokenStore {
    connection: Arc<RwLock<MultiplexedConnection>>,
}

impl RedisBannedTokenStore {
    pub fn new(connection: Arc<RwLock<MultiplexedConnection>>) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[instrument(name = "Adding banned token to Redis", skip_all)]
    async fn add_token(&self, token: &str) -> Result<(), TokenStoreError> {
        if token.is_empty() {
            return Err(TokenStoreError::MissingToken);
        }

        let key = get_key(token);
        let options = SetOptions::default().with_expiration(SetExpiry::EX(
            TOKEN_TTL_SECONDS
                .try_into()
                .map_err(|e: TryFromIntError| TokenStoreError::UnexpectedError(e.into()))?,
        ));

        let mut connection = self.connection.write().await;
        connection
            .set_options(key, true, options)
            .await
            .map_err(|e| TokenStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "Checking banned token in Redis", skip_all)]
    async fn check_token(&self, token: &str) -> Result<bool, TokenStoreError> {
        if token.is_empty() {
            return Err(TokenStoreError::MissingToken);
        }

        let key = get_key(token);
        let mut connection = self.connection.write().await;
        let banned = connection
            .exists(key)
            .await
            .map_err(|e| TokenStoreError::UnexpectedError(e.into()))?;

        Ok(banned)
    }
}

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
