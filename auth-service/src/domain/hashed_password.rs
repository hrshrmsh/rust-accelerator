use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;
use tracing::instrument;
use validator::ValidateRange;

use crate::domain::AuthAPIError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub async fn parse(pwd: String) -> Result<HashedPassword, AuthAPIError> {
        ValidateRange::validate_range(&pwd.len(), Some(8), None, None, None)
            .then_some(())
            .ok_or(AuthAPIError::InvalidCredentials)?;

        let pwd_hash = compute_password_hash(&pwd).await?;
        Ok(HashedPassword(pwd_hash))
    }

    pub fn parse_password_hash(hash: String) -> Result<HashedPassword, AuthAPIError> {
        PasswordHash::new(&hash).map_err(|_| AuthAPIError::UnexpectedError)?;
        Ok(HashedPassword(hash))
    }

    #[instrument(name = "Verify raw password", skip_all)]
    pub async fn verify_raw_password(&self, pwd_to_try: &str) -> Result<(), AuthAPIError> {
        let pwd_hash = self.as_ref().to_owned();
        let pwd_to_try = pwd_to_try.to_owned();

        spawn_blocking(move || {
            let expected_hash =
                PasswordHash::new(&pwd_hash).map_err(|_| AuthAPIError::UnexpectedError)?;
            Argon2::default()
                .verify_password(pwd_to_try.as_bytes(), &expected_hash)
                .map_err(|_| AuthAPIError::InvalidCredentials)
        })
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?
    }
}

#[instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(pwd: &str) -> Result<String, AuthAPIError> {
    let pwd = pwd.to_owned();

    spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).map_err(|_| AuthAPIError::UnexpectedError)?,
        )
        .hash_password(pwd.as_bytes(), &salt)
        .map_err(|_| AuthAPIError::UnexpectedError)?
        .to_string();

        Ok(password_hash)
    })
    .await
    .map_err(|_| AuthAPIError::UnexpectedError)?
}

impl AsRef<str> for HashedPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use fake::{Fake, faker::internet::en::Password};
    use quickcheck_macros::quickcheck;
    use rand::{SeedableRng, rngs::SmallRng};

    #[derive(Debug, Clone)]
    struct ValidArbitraryPassword(String);

    #[derive(Debug, Clone)]
    struct InvalidArbitraryPassword(String);

    impl quickcheck::Arbitrary for ValidArbitraryPassword {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = SmallRng::seed_from_u64(u64::arbitrary(g));
            let password = Password(8..32).fake_with_rng(&mut rng);
            Self(password)
        }
    }

    impl quickcheck::Arbitrary for InvalidArbitraryPassword {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = SmallRng::seed_from_u64(u64::arbitrary(g));
            let password = Password(0..8).fake_with_rng(&mut rng);
            Self(password)
        }
    }

    #[tokio::test]
    #[quickcheck(max_tests = 5)]
    async fn valid_passwords_parsed_successfully(password: ValidArbitraryPassword) -> bool {
        HashedPassword::parse(password.0).await.is_ok()
    }

    #[tokio::test]
    #[quickcheck(max_tests = 5)]
    async fn invalid_passwords_parsed_unsuccessfully(password: InvalidArbitraryPassword) -> bool {
        HashedPassword::parse(password.0).await.is_err()
    }

    #[test]
    fn can_parse_valid_argon2_hash() {
        let raw_password = "TestPassword123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        );

        let hash_string = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let hash_password = HashedPassword::parse_password_hash(hash_string.clone()).unwrap();

        assert_eq!(hash_password.as_ref(), hash_string.as_str());
        assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));
    }

    #[tokio::test]
    async fn can_verify_raw_password() {
        let raw_password = "TestPassword123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        );

        let hash_string = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let hash_password = HashedPassword::parse_password_hash(hash_string.clone()).unwrap();

        assert_eq!(hash_password.as_ref(), hash_string.as_str());
        assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));

        let result = hash_password.verify_raw_password(raw_password).await;
        assert!(result.is_ok())
    }
}
