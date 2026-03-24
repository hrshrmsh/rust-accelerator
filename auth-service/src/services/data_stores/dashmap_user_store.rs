use async_trait::async_trait;
use dashmap::DashMap;
use secrecy::SecretString;

use crate::domain::{Email, User, UserStore, UserStoreError};

#[derive(Clone, Default)]
pub struct DashMapUserStore {
    users: DashMap<Email, User>,
}

#[async_trait]
impl UserStore for DashMapUserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        self.users
            .get(email)
            .map(|u| u.clone())
            .ok_or_else(|| UserStoreError::UserNotFound)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &SecretString,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        user.password
            .verify_raw_password(password.clone())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::HashedPassword;

    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let store = DashMapUserStore::default();

        let user1 = User {
            email: Email::parse("a@b.com".into()).unwrap(),
            password: HashedPassword::parse("password".into()).await.unwrap(),
            requires_2fa: true,
        };

        assert_eq!(Ok(()), store.add_user(user1.clone()).await);
        assert_eq!(
            Err(UserStoreError::UserAlreadyExists),
            store.add_user(user1).await
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let store = DashMapUserStore::default();

        let user1 = User {
            email: Email::parse("a@b.com".into()).unwrap(),
            password: HashedPassword::parse("password".into()).await.unwrap(),
            requires_2fa: true,
        };
        store.add_user(user1.clone()).await.unwrap();

        assert_eq!(
            Ok(user1),
            store
                .get_user(&Email::parse("a@b.com".into()).unwrap())
                .await
        );
        assert_eq!(
            Err(UserStoreError::UserNotFound),
            store
                .get_user(&Email::parse("b@a.com".into()).unwrap())
                .await
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let store = DashMapUserStore::default();

        let user1 = User {
            email: Email::parse("a@b.com".into()).unwrap(),
            password: HashedPassword::parse("password".into()).await.unwrap(),
            requires_2fa: true,
        };
        store.add_user(user1.clone()).await.unwrap();

        assert_eq!(
            Ok(()),
            store.validate_user(&user1.email, &"password".into()).await
        );
        assert_eq!(
            Err(UserStoreError::InvalidCredentials),
            store
                .validate_user(&user1.email, &"wrong password".into())
                .await
        )
    }
}
