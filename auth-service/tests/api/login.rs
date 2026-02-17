use serde_json::json;

use auth_service::{ErrorResponse, utils::constants::JWT_COOKIE_NAME};

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;
    setup_users(&app).await;

    let response = app
        .post_login(&json!({
            "email": "azure@diamond.com",
            "password": "hunter22"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("no cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let test_cases = [
        json!({
            "password": "password123"
        }),
        json!({
            "email": "amazing@cool.com"
        }),
    ];

    for test_case in &test_cases {
        let response = app.post_login(test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let invalid_emails = ["", "don't have amerspand", "longstring12345?"];
    let invalid_passwords = ["", "1234567", "passwor"];

    for invalid_email in invalid_emails {
        let response = app
            .post_login(&json!({
                "email": invalid_email,
                "password": "validpassword",
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
            .post_login(&json!({
                "email": "totally@valid.com",
                "password": invalid_password,
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
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    setup_users(&app).await;

    let response = app
        .post_login(&json!({
            "email": "azure@diamond.com",
            "password": "wrongpassword"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.json::<ErrorResponse>().await.unwrap().error,
        "Authentication failed!"
    );

    let response = app
        .post_login(&json!({
            "email": "azure2@diamond.com",
            "password": "hunter22"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.json::<ErrorResponse>().await.unwrap().error,
        "Authentication failed!"
    );
}

// helper database
async fn setup_users(app: &TestApp) {
    let users = [
        json!({
            "email": "azure@diamond.com",
            "password": "hunter22",
            "requires2FA": false
        }),
        json!({
            "email": "cthon98@bash.org",
            "password": "7!superdupersecure!7",
            "requires2FA": false
        }),
    ];

    for user in &users {
        app.post_signup(user).await.error_for_status().unwrap();
    }
}
