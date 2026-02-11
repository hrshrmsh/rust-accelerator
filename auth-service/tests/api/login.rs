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
