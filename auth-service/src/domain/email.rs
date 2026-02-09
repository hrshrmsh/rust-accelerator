use std::str::FromStr;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidateEmail, ValidationError, ValidationErrors};

use crate::domain::AuthAPIError;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl FromStr for Email {
    type Err = AuthAPIError;

    fn from_str(email: &str) -> Result<Self, AuthAPIError> {
        let parsed = Email(email.to_string());

        parsed
            .validate()
            .map_err(|_| AuthAPIError::InvalidCredentials)?;
        Ok(parsed)
    }
}

impl Validate for Email {
    fn validate(&self) -> Result<(), ValidationErrors> {
        ValidateEmail::validate_email(&self.0)
            .then_some(())
            .ok_or_else(|| {
                let mut errs = ValidationErrors::new();
                errs.add("email", ValidationError::new("invalid email"));
                errs
            })
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck_macros::quickcheck;
    use rand::{rngs::SmallRng, SeedableRng};

    #[derive(Debug, Clone)]
    struct ValidArbitraryEmail(String);

    // not quite perfect but a true impl is too verbose & out of scope here
    impl quickcheck::Arbitrary for ValidArbitraryEmail {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = SmallRng::seed_from_u64(u64::arbitrary(g));
            let email: String = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck]
    fn valid_emails_parsed_successfully(email: ValidArbitraryEmail) -> bool {
        email.0.parse::<Email>().is_ok()
    }

    // negative variant (invalid emails) omitted
}
