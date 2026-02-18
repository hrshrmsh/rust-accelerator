use auth_service::utils::constants::JWT_COOKIE_NAME;
use serde_json::json;

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;

    app.post_signup(&json!({
        "email": "hello@world.com",
        "password": "password123",
        "requires2FA": false,
    }))
    .await
    .error_for_status()
    .unwrap();

    let login_response = app
        .post_login(&json!({
            "email": "hello@world.com",
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

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let response = app
        .post_verify_token(&json!({
            "weird": "stuff"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

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
