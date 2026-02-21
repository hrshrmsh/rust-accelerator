use async_trait::async_trait;

use crate::domain::{AuthAPIError, Email};

#[async_trait]
pub trait EmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        contents: &str,
    ) -> Result<(), AuthAPIError>;
}
