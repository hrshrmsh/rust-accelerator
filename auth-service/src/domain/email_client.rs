use async_trait::async_trait;
use color_eyre::Result;

use crate::domain::Email;

#[async_trait]
pub trait EmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, contents: &str) -> Result<()>;
}
