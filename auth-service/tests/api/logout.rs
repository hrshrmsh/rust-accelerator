use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Response;
use serde_json::json;

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;

    let _ = setup_user(&app).await;

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 200);
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
