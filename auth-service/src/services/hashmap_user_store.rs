use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        } else {
            self.users.insert((&user.email).clone(), user);
            Ok(())
        }
    }

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        self.users
            .get(email)
            .map(|u| u.clone())
            .ok_or_else(|| UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email)?;
        if user.password != password {
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
            email: String::from("a@b.com"),
            password: String::from("password"),
            requires_2fa: true,
        };

        assert_eq!(Ok(()), store.add_user(user1.clone()));
        assert_eq!(
            Err(UserStoreError::UserAlreadyExists),
            store.add_user(user1)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore {
            ..Default::default()
        };
        let user1 = User {
            email: String::from("a@b.com"),
            password: String::from("password"),
            requires_2fa: true,
        };
        store.add_user(user1.clone()).unwrap();

        assert_eq!(Ok(user1), store.get_user("a@b.com"));
        assert_eq!(Err(UserStoreError::UserNotFound), store.get_user("?"));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore {
            ..Default::default()
        };
        let user1 = User {
            email: String::from("a@b.com"),
            password: String::from("password"),
            requires_2fa: true,
        };
        store.add_user(user1.clone()).unwrap();

        assert_eq!(Ok(()), store.validate_user(&user1.email, &user1.password));
        assert_eq!(
            Err(UserStoreError::InvalidCredentials),
            store.validate_user(&user1.email, "wrong password")
        )
    }
}
