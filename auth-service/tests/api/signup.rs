use crate::helpers::TestApp;

use auth_service::routes::SignupResponse;
use serde_json::json;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let test_cases = [
        json! {
            {
                "password": "password123",
                "requires2FA": true,
            }
        },
        json! {
            {
                "email": random_email,
            }
        },
    ];

    for test_case in &test_cases {
        let response = app.post_signup(&test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;

    let random_emails = &[TestApp::get_random_email(), TestApp::get_random_email()];

    let test_cases = [
        json! {
            {
                "email": random_emails[0],
                "password": "password123",
                "requires2FA": true
            }
        },
        json! {
            {
                "requires2FA": true,
                "password": "123456",
                "email": random_emails[1]
            }
        },
    ];

    for test_case in &test_cases {
        let response = app.post_signup(&test_case).await;
        let expected_response = SignupResponse {
            message: String::from("User created successfully!")
        };

        assert_eq!(response.status().as_u16(), 201);
        assert_eq!(response.json::<SignupResponse>().await.unwrap(), expected_response)
    }
}
