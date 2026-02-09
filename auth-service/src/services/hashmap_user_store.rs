use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        } else {
            self.users.insert((&user.email).clone(), user);
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
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        if user.password != *password {
            Err(UserStoreError::InvalidCredentials)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore {
            ..Default::default()
        };
        let user1 = User {
            email: "a@b.com".parse().unwrap(),
            password: "password".parse().unwrap(),
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
        let mut store = HashmapUserStore {
            ..Default::default()
        };
        let user1 = User {
            email: "a@b.com".parse().unwrap(),
            password: "password".parse().unwrap(),
            requires_2fa: true,
        };
        store.add_user(user1.clone()).await.unwrap();

        assert_eq!(Ok(user1), store.get_user(&"a@b.com".parse().unwrap()).await);
        assert_eq!(
            Err(UserStoreError::UserNotFound),
            store.get_user(&"b@a.com".parse().unwrap()).await
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore {
            ..Default::default()
        };
        let user1 = User {
            email: "a@b.com".parse().unwrap(),
            password: "password".parse().unwrap(),
            requires_2fa: true,
        };
        store.add_user(user1.clone()).await.unwrap();

        assert_eq!(
            Ok(()),
            store.validate_user(&user1.email, &user1.password).await
        );
        assert_eq!(
            Err(UserStoreError::InvalidCredentials),
            store
                .validate_user(&user1.email, &"wrong password".parse().unwrap())
                .await
        )
    }
}
