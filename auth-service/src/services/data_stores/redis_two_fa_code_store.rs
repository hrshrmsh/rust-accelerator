use std::sync::Arc;

use async_trait::async_trait;
use redis::{AsyncTypedCommands, SetExpiry, SetOptions, aio::MultiplexedConnection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::instrument;

use crate::domain::{
    AuthAPIError, Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError,
};

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

pub struct RedisTwoFACodeStore {
    connection: Arc<RwLock<MultiplexedConnection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(connection: Arc<RwLock<MultiplexedConnection>>) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    #[instrument(name = "Adding 2fa code to redis", skip_all)]
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        // shouldn't hit these paths; written for insurance
        let key = get_key(&email);
        let data = TwoFATuple(
            login_attempt_id.as_ref().to_owned(),
            code.as_ref().to_owned(),
        );
        let data = serde_json::to_string(&data)
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        let options = SetOptions::default().with_expiration(SetExpiry::EX(TEN_MINUTES_IN_SECONDS));
        let mut connection = self.connection.write().await;
        connection
            .set_options(key, data, options)
            .await
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "Removing 2fa code from redis", skip_all)]
    async fn remove_code(&self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);

        let mut connection = self.connection.write().await;
        connection
            .del(key)
            .await
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[instrument(name = "Getting 2fa code from redis", skip_all)]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(email);

        // if a vec is stored for the key -> unexpected error
        // if nil is returned -> not found
        let mut connection = self.connection.write().await;
        let data = connection
            .get(key)
            .await
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)?;

        let data: TwoFATuple = serde_json::from_str(&data)
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;
        let (login_attempt_id, two_fa_code) = (
            LoginAttemptId::parse(data.0.into())
                .map_err(|e: AuthAPIError| TwoFACodeStoreError::UnexpectedError(e.into()))?,
            TwoFACode::parse(data.1.into())
                .map_err(|e: AuthAPIError| TwoFACodeStoreError::UnexpectedError(e.into()))?,
        );

        Ok((login_attempt_id, two_fa_code))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
