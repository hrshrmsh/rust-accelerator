use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use secrecy::{ExposeSecret, SecretString};
use tokio::task::spawn_blocking;
use tracing::{Span, instrument};
use validator::ValidateRange;

use crate::domain::AuthAPIError;

#[derive(Clone, Debug)]
pub struct HashedPassword(SecretString);

impl PartialEq for HashedPassword {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Eq for HashedPassword {}

impl HashedPassword {
    #[instrument(name = "Parse hashed password", skip_all)]
    pub async fn parse(pwd: SecretString) -> Result<HashedPassword> {
        ValidateRange::validate_range(&pwd.expose_secret().len(), Some(8), None, None, None)
            .then_some(())
            .ok_or(eyre!("invalid password"))?;

        let pwd_hash = compute_password_hash(&pwd).await?;
        Ok(HashedPassword(pwd_hash))
    }

    pub fn parse_password_hash(hash: SecretString) -> Result<HashedPassword> {
        PasswordHash::new(hash.expose_secret().as_ref())?;
        Ok(HashedPassword(hash))
    }

    #[instrument(name = "Verify raw password", skip_all)]
    pub async fn verify_raw_password(&self, pwd_to_try: SecretString) -> Result<()> {
        let current_span = Span::current();
        let pwd_hash = self.as_ref().to_owned();
        let pwd_to_try = pwd_to_try.to_owned();

        spawn_blocking(move || {
            current_span.in_scope(|| {
                let expected_hash = PasswordHash::new(&pwd_hash)?;
                Argon2::default()
                    .verify_password(pwd_to_try.expose_secret().as_bytes(), &expected_hash)
                    .wrap_err("failed to verify password hash")
            })
        })
        .await?
    }
}

#[instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(pwd: &SecretString) -> Result<SecretString> {
    let current_span = Span::current();
    let pwd = pwd.to_owned();

    spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)
                    .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?,
            )
            .hash_password(pwd.expose_secret().as_bytes(), &salt)
            .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?
            .to_string();

            Ok(SecretString::new(password_hash.into_boxed_str()))
        })
    })
    .await?
}

impl AsRef<str> for HashedPassword {
    fn as_ref(&self) -> &str {
        &self.0.expose_secret()
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
        HashedPassword::parse(SecretString::new(password.0.into_boxed_str()))
            .await
            .is_ok()
    }

    #[tokio::test]
    #[quickcheck(max_tests = 5)]
    async fn invalid_passwords_parsed_unsuccessfully(password: InvalidArbitraryPassword) -> bool {
        HashedPassword::parse(SecretString::new(password.0.into_boxed_str()))
            .await
            .is_err()
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

        let hash_password = HashedPassword::parse_password_hash(SecretString::new(
            hash_string.clone().into_boxed_str(),
        ))
        .unwrap();

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

        let hash_password = HashedPassword::parse_password_hash(SecretString::new(
            hash_string.clone().into_boxed_str(),
        ))
        .unwrap();

        assert_eq!(hash_password.as_ref(), hash_string.as_str());
        assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));

        let result = hash_password
            .verify_raw_password(SecretString::new(raw_password.to_owned().into_boxed_str()))
            .await;
        assert!(result.is_ok())
    }
}
