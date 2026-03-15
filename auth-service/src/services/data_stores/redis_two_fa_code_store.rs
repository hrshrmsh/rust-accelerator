use std::sync::Arc;

use async_trait::async_trait;
use redis::{AsyncTypedCommands, SetExpiry, SetOptions, aio::MultiplexedConnection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

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
        let data =
            serde_json::to_string(&data).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let options = SetOptions::default().with_expiration(SetExpiry::EX(TEN_MINUTES_IN_SECONDS));
        let mut connection = self.connection.write().await;
        connection
            .set_options(key, data, options)
            .await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn remove_code(&self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);

        let mut connection = self.connection.write().await;
        connection
            .del(key)
            .await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

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
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)?;

        let data: TwoFATuple =
            serde_json::from_str(&data).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        let (login_attempt_id, two_fa_code) = (
            data.0
                .parse()
                .map_err(|_| TwoFACodeStoreError::UnexpectedError)?,
            data.1
                .parse()
                .map_err(|_| TwoFACodeStoreError::UnexpectedError)?,
        );

        Ok((login_attempt_id, two_fa_code))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
