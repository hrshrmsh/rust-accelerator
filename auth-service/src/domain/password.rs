use std::str::FromStr;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidateRange, ValidationError, ValidationErrors};

use crate::domain::AuthAPIError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password(String);

impl FromStr for Password {
    type Err = AuthAPIError;

    fn from_str(password: &str) -> Result<Self, AuthAPIError> {
        let parsed = Password(password.to_string());

        parsed
            .validate()
            .map_err(|_| AuthAPIError::InvalidCredentials)?;
        Ok(parsed)
    }
}

impl Validate for Password {
    fn validate(&self) -> Result<(), ValidationErrors> {
        ValidateRange::validate_range(&self.0.len(), Some(8), None, None, None)
            .then_some(())
            .ok_or_else(|| {
                let mut errs = ValidationErrors::new();
                errs.add("password", ValidationError::new("invalid password"));
                errs
            })
    }
}

impl AsRef<str> for Password {
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

    #[quickcheck]
    fn valid_passwords_parsed_successfully(password: ValidArbitraryPassword) -> bool {
        password.0.parse::<Password>().is_ok()
    }

    #[quickcheck]
    fn invalid_passwords_parsed_unsuccessfully(password: InvalidArbitraryPassword) -> bool {
        password.0.parse::<Password>().is_err()
    }
}
