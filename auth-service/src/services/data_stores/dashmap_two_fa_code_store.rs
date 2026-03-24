use async_trait::async_trait;
use color_eyre::eyre::eyre;
use dashmap::DashMap;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

#[derive(Debug, Default)]
pub struct DashMapTwoFACodeStore {
    codes: DashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait]
impl TwoFACodeStore for DashMapTwoFACodeStore {
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes
            .remove(email)
            .ok_or_else(|| {
                TwoFACodeStoreError::UnexpectedError(eyre!("failed to remove 2fa code from store"))
            })
            .map(|_| ())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .map(|e| e.clone())
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Email, LoginAttemptId, TwoFACode};

    #[tokio::test]
    async fn test_add_code_and_get() {
        let store = DashMapTwoFACodeStore::default();

        let email = Email::parse("test@example.com".into()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::parse("123456".into()).unwrap();

        store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await
            .unwrap();

        let result = store.get_code(&email).await.unwrap();
        assert_eq!(result, (login_attempt_id, code));
    }

    #[tokio::test]
    async fn test_add_multiple_distinct_emails() {
        let store = DashMapTwoFACodeStore::default();

        let email1 = Email::parse("test1@example.com".into()).unwrap();
        let login_id1 = LoginAttemptId::default();
        let code1 = TwoFACode::parse("123456".into()).unwrap();

        let email2 = Email::parse("test2@example.com".into()).unwrap();
        let login_id2 = LoginAttemptId::default();
        let code2 = TwoFACode::parse("654321".into()).unwrap();

        store
            .add_code(email1.clone(), login_id1.clone(), code1.clone())
            .await
            .unwrap();
        store
            .add_code(email2.clone(), login_id2.clone(), code2.clone())
            .await
            .unwrap();

        assert_eq!(store.get_code(&email1).await.unwrap(), (login_id1, code1));
        assert_eq!(store.get_code(&email2).await.unwrap(), (login_id2, code2));
    }

    #[tokio::test]
    async fn test_add_code_overwrites_existing() {
        let store = DashMapTwoFACodeStore::default();

        let email = Email::parse("test@example.com".into()).unwrap();
        let login_id1 = LoginAttemptId::default();
        let code1 = TwoFACode::parse("123456".into()).unwrap();

        let login_id2 = LoginAttemptId::default();
        let code2 = TwoFACode::parse("654321".into()).unwrap();

        store
            .add_code(email.clone(), login_id1.clone(), code1.clone())
            .await
            .unwrap();
        assert_eq!(store.get_code(&email).await.unwrap(), (login_id1, code1));

        store
            .add_code(email.clone(), login_id2.clone(), code2.clone())
            .await
            .unwrap();
        assert_eq!(store.get_code(&email).await.unwrap(), (login_id2, code2));
    }

    #[tokio::test]
    async fn test_remove_existing() {
        let store = DashMapTwoFACodeStore::default();

        let email = Email::parse("test@example.com".into()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::parse("123456".into()).unwrap();

        store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await
            .unwrap();

        store.remove_code(&email).await.unwrap();
        assert_eq!(
            store.get_code(&email).await,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        )
    }

    #[tokio::test]
    async fn test_remove_nonexistent() {
        let store = DashMapTwoFACodeStore::default();

        let email = Email::parse("test@example.com".into()).unwrap();

        let result = store.remove_code(&email).await;
        assert_eq!(
            result.unwrap_err(),
            TwoFACodeStoreError::UnexpectedError(eyre!("test"))
        );
    }

    #[tokio::test]
    async fn test_remove_twice() {
        let store = DashMapTwoFACodeStore::default();

        let email = Email::parse("test@example.com".into()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::parse("123456".into()).unwrap();

        store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await
            .unwrap();

        store.remove_code(&email).await.unwrap();
        assert_eq!(
            store.get_code(&email).await,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        );

        let result = store.remove_code(&email).await;
        assert_eq!(
            result.unwrap_err(),
            TwoFACodeStoreError::UnexpectedError(eyre!("test"))
        );
    }
}
