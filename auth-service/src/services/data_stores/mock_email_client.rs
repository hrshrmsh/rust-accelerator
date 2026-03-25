use async_trait::async_trait;
use color_eyre::Result;
use tracing::debug;

use crate::domain::{Email, EmailClient};

pub struct MockEmailClient;

#[async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, contents: &str) -> Result<()> {
        debug!("[Notification]: Email sent!");
        debug!("[Sender]: You (someone@somewhere.com)");
        debug!("[Recipient]: {}", recipient.as_ref());
        debug!("[Subject]: {}", subject);
        debug!("[Contents]: {}", contents);
        Ok(())
    }
}
