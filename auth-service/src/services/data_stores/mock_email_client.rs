use async_trait::async_trait;

use crate::domain::{AuthAPIError, Email, EmailClient};

pub struct MockEmailClient;

#[async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        contents: &str,
    ) -> Result<(), AuthAPIError> {
        println!("[Notification]: Email sent!");
        println!("[Sender]: You (someone@somewhere.com)");
        println!("[Recipient]: {}", recipient.as_ref());
        println!("[Subject]: {}", subject);
        println!("[Contents]: {}", contents);
        Ok(())
    }
}
