use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::domain::AuthAPIError;

#[derive(Debug, Clone)]
pub struct LoginAttemptId(SecretString);

impl LoginAttemptId {
    pub fn parse(s: SecretString) -> Result<Self, AuthAPIError> {
        let attempt = LoginAttemptId(s);

        attempt
            .validate()
            .map_err(|_| AuthAPIError::InvalidLoginAttemptId)?;
        Ok(attempt)
    }
}

impl Validate for LoginAttemptId {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let id = &self.0.expose_secret();

        // parse_str may be deprecated soon
        if Uuid::try_parse(id).is_err() {
            let mut errors = ValidationErrors::new();
            errors.add(
                "id",
                ValidationError::new("invalid uuid").with_message("must be a valid UUID".into()),
            );
            return Err(errors);
        }

        Ok(())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0.expose_secret()
    }
}

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(Uuid::new_v4().to_string().into())
    }
}
