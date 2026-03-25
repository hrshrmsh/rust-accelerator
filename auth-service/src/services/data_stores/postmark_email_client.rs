use async_trait::async_trait;
use color_eyre::Result;
use reqwest::{Client, Url};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

use crate::domain::{Email, EmailClient};

pub struct PostmarkEmailClient {
    http_client: Client,
    base_url: SecretString,
    sender: Email,
    auth_token: SecretString,
}

impl PostmarkEmailClient {
    pub fn new(
        http_client: Client,
        base_url: SecretString,
        sender: Email,
        auth_token: SecretString,
    ) -> Self {
        Self {
            http_client,
            base_url,
            sender,
            auth_token,
        }
    }
}

#[async_trait]
impl EmailClient for PostmarkEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, contents: &str) -> Result<()> {
        // parse the base url & join it w/ email endpoint
        let url = Url::parse(self.base_url.expose_secret())?.join("/email")?;

        // create the request body for sending the email
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: contents,
            text_body: contents,
            message_stream: MESSAGE_STREAM,
        };

        // build http post request
        let request = self
            .http_client
            .post(url)
            .header(POSTMARK_AUTH_HEADER, self.auth_token.expose_secret())
            .json(&request_body);

        request.send().await?.error_for_status()?;

        Ok(())
    }
}

const MESSAGE_STREAM: &str = "outbound";
const POSTMARK_AUTH_HEADER: &str = "X-Postmark-Server-Token";

// defines email request body structure
// for more info about the request structure, check API docs: https://postmarkapp.com/developer/user-guide/send-email-with-api
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
    message_stream: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::utils::constants::test;

    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use super::PostmarkEmailClient;

    // Helper function to generate a test subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    // Helper function to generate test content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    // Helper function to generate a test email
    fn email() -> Email {
        Email::parse(SecretString::new(
            SafeEmail().fake::<String>().into_boxed_str(),
        ))
        .unwrap()
    }

    // Helper function to create a test email client
    fn email_client(base_url: String) -> PostmarkEmailClient {
        let http_client = Client::builder()
            .timeout(test::email_client::TIMEOUT)
            .build()
            .unwrap();
        PostmarkEmailClient::new(
            http_client,
            base_url.into(),
            email(),
            Faker.fake::<String>().into(),
        )
    }

    // Custom matcher to validate the email request body
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
                    && body.get("MessageStream").is_some()
            } else {
                false
            }
        }
    }

    // Test to ensure the email client sends the expected request
    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // Set up the mock server to expect a specific request
        Mock::given(header_exists(POSTMARK_AUTH_HEADER))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Execute the send_email function and check the outcome
        let outcome = email_client
            .send_email(&email(), &subject(), &content())
            .await;

        assert!(outcome.is_ok());
    }

    // Test to handle server error responses
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // Set up the mock server to respond with a 500 error
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Execute the send_email function and check the outcome
        let outcome = email_client
            .send_email(&email(), &subject(), &content())
            .await;

        assert!(outcome.is_err());
    }

    // Test to handle request timeouts
    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // Set up the mock server to delay the response
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180)); // 3 minutes delay
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Execute the send_email function and check the outcome
        let outcome = email_client
            .send_email(&email(), &subject(), &content())
            .await;

        assert!(outcome.is_err());
    }
}
