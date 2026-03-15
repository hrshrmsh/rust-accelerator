use auth_service::utils::constants::JWT_COOKIE_NAME;
use serde_json::json;
use test_context::test_context;

use crate::helpers::TestApp;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_valid_token(app: &mut TestApp) {
    let email = TestApp::get_random_email();
    app.post_signup(&json!({
        "email": &email,
        "password": "password123",
        "requires2FA": false,
    }))
    .await
    .error_for_status()
    .unwrap();

    let login_response = app
        .post_login(&json!({
            "email": &email,
            "password": "password123"
        }))
        .await
        .error_for_status()
        .unwrap();

    let cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("no jwt cookie found in /login route");

    let verify_response = app
        .post_verify_token(&json!({
            "token": cookie.value()
        }))
        .await;

    assert_eq!(verify_response.status().as_u16(), 200);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_input(app: &mut TestApp) {
    let response = app
        .post_verify_token(&json!({
            "weird": "stuff"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 422);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_invalid_token(app: &mut TestApp) {
    let response = app
        .post_verify_token(&json!({
            "token": format!(
                "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
                JWT_COOKIE_NAME
            )
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
}
