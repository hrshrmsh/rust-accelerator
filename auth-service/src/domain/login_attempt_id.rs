use std::str::FromStr;

use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::domain::AuthAPIError;

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl FromStr for LoginAttemptId {
    type Err = AuthAPIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = LoginAttemptId(s.to_string());

        parsed
            .validate()
            .map_err(|_| AuthAPIError::InvalidLoginAttemptId)?;
        Ok(parsed)
    }
}

impl Validate for LoginAttemptId {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let id = &self.0;

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
        &self.0
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(Uuid::new_v4().to_string())
    }
}
