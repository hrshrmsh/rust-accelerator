use serde_json::json;

use auth_service::{
    ErrorResponse, domain::TwoFACodeStore, routes::LoginResponse, utils::constants::JWT_COOKIE_NAME,
};
use test_context::test_context;

use crate::helpers::TestApp;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled(app: &mut TestApp) {
    let (email, password) = setup_users(&app, false).await;

    let response = app
        .post_login(&json!({
            "email": email,
            "password": password
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("no cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled(app: &mut TestApp) {
    let (email, password) = setup_users(&app, true).await;

    let response = app
        .post_login(&json!({
            "email": email.clone(),
            "password": password
        }))
        .await;
    assert_eq!(response.status().as_u16(), 206);

    let code = app
        .two_fa_code_store
        .get_code(&email.parse().unwrap())
        .await
        .unwrap();
    let parsed_response = response.json::<LoginResponse>().await.unwrap();

    assert_eq!(parsed_response.message, "2fa required!");
    assert_eq!(parsed_response.login_attempt_id, code.0.as_ref());
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_invalid_input(app: &mut TestApp) {
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

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(app: &mut TestApp) {
    let (email, password) = setup_users(&app, false).await;
    let (mut wrong_email, mut wrong_password) = (email.clone(), password.clone());

    wrong_email.push('A');
    wrong_password.push('A');

    let response = app
        .post_login(&json!({
            "email": email,
            "password": wrong_password
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.json::<ErrorResponse>().await.unwrap().error,
        "Authentication failed!"
    );

    let response = app
        .post_login(&json!({
            "email": wrong_email,
            "password": password
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.json::<ErrorResponse>().await.unwrap().error,
        "Authentication failed!"
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_credentials(app: &mut TestApp) {
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

// helper database
async fn setup_users(app: &TestApp, requires_2fa: bool) -> (String, String) {
    let email = TestApp::get_random_email();
    let password = "password123";
    let user = json!({
        "email": &email.to_owned(),
        "password": password.to_owned(),
        "requires2FA": requires_2fa
    });

    app.post_signup(&user).await.error_for_status().unwrap();
    (email, password.to_owned())
}
