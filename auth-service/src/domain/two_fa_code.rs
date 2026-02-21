use std::str::FromStr;

use validator::{Validate, ValidationError, ValidationErrors};

use crate::domain::AuthAPIError;

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl FromStr for TwoFACode {
    type Err = AuthAPIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = TwoFACode(s.to_string());

        parsed
            .validate()
            .map_err(|_| AuthAPIError::Invalid2FACode)?;
        Ok(parsed)
    }
}

impl Validate for TwoFACode {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let code = &self.0;

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
        &self.0
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let val = rand::random_range(0..1_000_000);
        Self(format!("{:06}", val))
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
