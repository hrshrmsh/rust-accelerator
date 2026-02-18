use async_trait::async_trait;
use dashmap::DashSet;

use crate::domain::{BannedTokenStore, TokenStoreError};

#[derive(Clone, Debug, Default)]
pub struct HashSetTokenStore {
    banned_tokens: DashSet<String>,
}

impl HashSetTokenStore {
    pub fn new() -> Self {
        Self {
            banned_tokens: DashSet::new(),
        }
    }
}

#[async_trait]
impl BannedTokenStore for HashSetTokenStore {
    async fn add_token(&self, token: &str) -> Result<(), TokenStoreError> {
        if token.is_empty() {
            Err(TokenStoreError::MissingToken)
        } else {
            self.banned_tokens.insert(token.to_string());
            Ok(())
        }
    }

    async fn check_token(&self, token: &str) -> Result<bool, TokenStoreError> {
        if token.is_empty() {
            Err(TokenStoreError::MissingToken)
        } else {
            Ok(self.banned_tokens.contains(token))
        }
    }
}

// for tests, this token store should not validate tokens - that duplicates responsibility
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token_success_and_idempotent() {
        let store = HashSetTokenStore::new();
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.KMUFsIDTnFmyG3nMiGM6H9FNFUROf3wh7SmqJp-QV30";

        assert_eq!(Ok(()), store.add_token(token).await);
        // adding again is fine
        assert_eq!(Ok(()), store.add_token(token).await);
    }

    #[tokio::test]
    async fn test_add_empty_token_fails() {
        let store = HashSetTokenStore::new();

        assert_eq!(
            Err(TokenStoreError::MissingToken),
            store.add_token("").await
        );
    }

    #[tokio::test]
    async fn test_check_token() {
        let store = HashSetTokenStore::new();
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.KMUFsIDTnFmyG3nMiGM6H9FNFUROf3wh7SmqJp-QV30";

        assert_eq!(Ok(false), store.check_token(token).await);

        store.add_token(token).await.unwrap();
        assert_eq!(Ok(true), store.check_token(token).await);
        assert_eq!(Ok(false), store.check_token("some other token").await);
    }

    #[tokio::test]
    async fn test_check_empty_token_fails() {
        let store = HashSetTokenStore::new();

        assert_eq!(
            Err(TokenStoreError::MissingToken),
            store.check_token("").await
        );
    }
}
