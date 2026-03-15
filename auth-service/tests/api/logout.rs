use auth_service::{domain::BannedTokenStore, utils::constants::JWT_COOKIE_NAME};
use reqwest::Response;
use serde_json::json;
use test_context::test_context;

use crate::helpers::TestApp;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie(app: &mut TestApp) {
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
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing(app: &mut TestApp) {
    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row(app: &mut TestApp) {
    setup_user(&app).await.error_for_status().ok();
    app.post_logout().await.error_for_status().ok();
    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_invalid_token(app: &mut TestApp) {
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

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_banned_token(app: &mut TestApp) {
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
    );
}

async fn setup_user(app: &TestApp) -> Response {
    let email = TestApp::get_random_email();
    app.post_signup(&json!({
        "email": email.to_owned(),
        "password": "password123",
        "requires2FA": false
    }))
    .await
    .error_for_status()
    .unwrap();

    app.post_login(&json!({
        "email": email.to_owned(),
        "password": "password123",
    }))
    .await
}
