use serde_json::json;

use crate::helpers::TestApp;

#[tokio::test]
async fn login_returns_ok() {
    let app = TestApp::new().await;

    app.post_signup(&json!({
        "email": "azure@diamond.com",
        "password": "hunter22",
        "requires2FA": false
    }))
    .await
    .error_for_status()
    .unwrap();

    let response = app
        .post_login(&json!({
            "email": "azure@diamond.com",
            "password": "hunter22"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);
}
