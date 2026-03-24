use std::str::FromStr;

use secrecy::{ExposeSecret, SecretString};
use validator::{Validate, ValidationError, ValidationErrors};

use crate::domain::AuthAPIError;

#[derive(Clone, Debug)]
pub struct TwoFACode(SecretString);

impl TwoFACode {
    pub fn parse(s: SecretString) -> Result<Self, AuthAPIError> {
        let attempt = TwoFACode(s);

        attempt
            .validate()
            .map_err(|_| AuthAPIError::Invalid2FACode)?;
        Ok(attempt)
    }
}

impl Validate for TwoFACode {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let code = &self.0.expose_secret();

        if code.len() != 6 || u32::from_str(code).is_err() {
            let mut errors = ValidationErrors::new();
            errors.add(
                "code",
                ValidationError::new("invalid 2fa code")
                    .with_message("must be exactly 6 digits".into()),
            );
            return Err(errors);
        }

        Ok(())
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0.expose_secret()
    }
}

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let val = rand::random_range(0..1_000_000);
        Self(format!("{:06}", val).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // written to catch annoying formatting errors and prevent them from reoccuring
    #[test]
    fn default_is_valid() {
        let code = TwoFACode::default();
        code.validate().unwrap();
    }
}
