use auth_service::{domain::BannedTokenStore, utils::constants::JWT_COOKIE_NAME};
use reqwest::Response;
use serde_json::json;

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;

    let jwt_token = setup_user(&app)
        .await
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("no jwt given from /login")
        .value()
        .to_string();

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 200);
    assert!(
        app.banned_token_store
            .check_token(&jwt_token)
            .await
            .unwrap()
    )
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;

    setup_user(&app).await.error_for_status().ok();
    app.post_logout().await.error_for_status().ok();
    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &"http://127.0.0.1".parse().expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let app = TestApp::new().await;

    let jwt_token = setup_user(&app)
        .await
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("no jwt given from /login")
        .value()
        .to_string();

    let _ = app.post_logout().await;
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME, &jwt_token,
        ),
        &"http://127.0.0.1".parse().expect("Failed to parse URL"),
    );
    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 401);
    assert!(
        app.banned_token_store
            .check_token(&jwt_token)
            .await
            .unwrap()
    )
}

async fn setup_user(app: &TestApp) -> Response {
    app.post_signup(&json!({
        "email": "sample@example.com",
        "password": "password123",
        "requires2FA": true
    }))
    .await
    .error_for_status()
    .unwrap();

    app.post_login(&json!({
        "email": "sample@example.com",
        "password": "password123",
    }))
    .await
}
