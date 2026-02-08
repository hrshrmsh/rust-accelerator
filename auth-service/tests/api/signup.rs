use crate::helpers::TestApp;

use auth_service::{routes::SignupResponse, ErrorResponse};
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
            message: String::from("User created successfully!"),
        };

        assert_eq!(response.status().as_u16(), 201);
        assert_eq!(
            response.json::<SignupResponse>().await.unwrap(),
            expected_response
        )
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let invalid_emails = &["", "don't have amerspand", "longstring12345?"];
    let invalid_passwords = &["", "1234567", "passwor"];

    for invalid_email in invalid_emails {
        let response = app
            .post_signup(&json!({
                "email": invalid_email,
                "password": "validpassword",
                "requires_2fa": true,
            }))
            .await;

        assert_eq!(response.status().as_u16(), 400);
        assert_eq!(
            response.json::<ErrorResponse>().await.unwrap().error,
            "Invalid credentials!"
        );
    }
    for invalid_password in invalid_passwords {
        let response = app
            .post_signup(&json!({
                "email": "valid@email.com",
                "password": invalid_password,
                "requires_2fa": true,
            }))
            .await;

        assert_eq!(response.status().as_u16(), 400);
        assert_eq!(
            response.json::<ErrorResponse>().await.unwrap().error,
            "Invalid credentials!"
        );
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app = TestApp::new().await;
    // Call the signup route twice. The second request should fail with a 409 HTTP status code
    let user = json!({
        "email": "hello@world.com",
        "password": "password123",
        "requires_2fa": true,
    });

    app.post_signup(&user).await.error_for_status().unwrap();

    let response = app.post_signup(&user).await;
    assert_eq!(response.status().as_u16(), 409);
    assert_eq!(
        response.json::<ErrorResponse>().await.unwrap().error,
        "User already exists"
    );
}
