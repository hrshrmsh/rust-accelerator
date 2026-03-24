use std::hash::Hash;

use color_eyre::{Result, eyre::eyre};
use secrecy::{ExposeSecret, SecretString};
use validator::{ValidateEmail, ValidationError, ValidationErrors};

#[derive(Clone, Debug)]
pub struct Email(SecretString);

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Eq for Email {}

impl Email {
    pub fn parse(s: SecretString) -> Result<Email> {
        if validate_email(&s).is_ok() {
            Ok(Self(s))
        } else {
            Err(eyre!(format!("{} is not a valid email", s.expose_secret())))
        }
    }
}

fn validate_email(email: &SecretString) -> Result<(), ValidationErrors> {
    ValidateEmail::validate_email(&email.expose_secret())
        .then_some(())
        .ok_or_else(|| {
            let mut errs = ValidationErrors::new();
            errs.add("email", ValidationError::new("invalid email"));
            errs
        })
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0.expose_secret()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use fake::{Fake, faker::internet::en::SafeEmail};
    use quickcheck_macros::quickcheck;
    use rand::{SeedableRng, rngs::SmallRng};

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
        Email::parse(email.0.into()).is_ok()
    }

    // negative variant (invalid emails) omitted
}
