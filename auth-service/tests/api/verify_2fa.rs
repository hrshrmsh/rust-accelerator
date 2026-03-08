use crate::helpers::TestApp;
use auth_service::domain::TwoFACodeStore;
use auth_service::routes::LoginResponse;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_if_correct_code(app: &mut TestApp) {
    // Make sure to assert the auth cookie gets set
    let email = "test@example.com";
    let password = "password123";

    app.post_signup(&json!({
        "email": email,
        "password": password,
        "requires2FA": true
    }))
    .await
    .error_for_status()
    .unwrap();

    let login_response = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await
        .error_for_status()
        .unwrap();
    assert!(
        login_response
            .cookies()
            .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
            .is_none()
    );

    let login: LoginResponse = login_response.json().await.unwrap();

    let code = app
        .two_fa_code_store
        .get_code(&email.parse().unwrap())
        .await
        .unwrap()
        .1;

    let verify_body = json!({
        "email": email,
        "loginAttemptId": login.login_attempt_id,
        "2FACode": code.as_ref()
    });

    let response = app.post_verify_2fa(&verify_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("no auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_input(app: &mut TestApp) {
    let test_cases = [
        json!({}),
        json!({
            "email_address": "user@example.com",
            "login_attempt": "not-a-uuid",
            "code": "123456"
        }),
        json!({
            "email": 12345,
            "loginAttemptId": 67890,
            "2FACode": true
        }),
        json!("just a string"),
        json!(null),
        json!({
            "email": "user@example.com",
            "twoFaCode": "123456"
        }),
    ];

    for test_case in &test_cases {
        let response = app.post_verify_2fa(test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_invalid_input(app: &mut TestApp) {
    let test_cases = [
        json!({
            "email": "invalid",
            "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "123456"
        }),
        json!({
            "email": "user@example.com",
            "loginAttemptId": "invalid",
            "2FACode": "123456"
        }),
        json!({
            "email": "user@example.com",
            "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "12345"
        }),
        json!({
            "email": "user@example.com",
            "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "1234567"
        }),
        json!({
            "email": "user@example.com",
            "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
            "2FACode": "12345a"
        }),
    ];

    for test_case in &test_cases {
        let response = app.post_verify_2fa(test_case).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(app: &mut TestApp) {
    let email = "test@example.com".to_string();
    let password = "password123".to_string();

    app.post_signup(&json!({
        "email": email.as_str(),
        "password": password.as_str(),
        "requires2FA": true
    }))
    .await
    .error_for_status()
    .unwrap();

    let login_response = app
        .post_login(&json!({
            "email": email.as_str(),
            "password": password.as_str(),
        }))
        .await
        .error_for_status()
        .unwrap();

    let login: LoginResponse = login_response.json().await.unwrap();

    // can't pick a random #, so this is a work around
    let wrong_code = format!(
        "{:06}",
        (app.two_fa_code_store
            .get_code(&"test@example.com".parse().unwrap())
            .await
            .unwrap()
            .1
            .as_ref()
            .parse::<u32>()
            .unwrap()
            + 1)
            % 1_000_000u32
    );

    let verify_body = json!({
        "email": email.as_str(),
        "loginAttemptId": login.login_attempt_id,
        "2FACode": wrong_code.as_str()
    });

    let response = app.post_verify_2fa(&verify_body).await;

    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed for input: {:?}",
        verify_body
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_old_code(app: &mut TestApp) {
    let email = "user@example.com".to_string();
    let password = "password123".to_string();

    app.post_signup(&json!({
        "email": &email,
        "password": &password,
        "requires2FA": true
    }))
    .await
    .error_for_status()
    .unwrap();

    let login_body = json!({
        "email": &email,
        "password": &password
    });
    let login_response: LoginResponse = app.post_login(&login_body).await.json().await.unwrap();
    app.post_login(&login_body).await;

    let verify_response = app
        .post_verify_2fa(&json!({
            "email": &email,
            "loginAttemptId": &login_response.login_attempt_id,
            "2FACode": app.two_fa_code_store.get_code(&email.parse().unwrap()).await.unwrap().1.as_ref(),
        }))
        .await;

    assert_eq!(
        verify_response.status().as_u16(),
        401,
        "Failed for input {:?}",
        verify_response
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_same_code_twice(app: &mut TestApp) {
    let email = "test@example.com";
    let password = "password123";

    app.post_signup(&json!({
        "email": email,
        "password": password,
        "requires2FA": true
    }))
    .await
    .error_for_status()
    .unwrap();

    let login_response = app
        .post_login(&json!({
            "email": email,
            "password": password,
        }))
        .await
        .error_for_status()
        .unwrap();

    let login: LoginResponse = login_response.json().await.unwrap();

    let code = app
        .two_fa_code_store
        .get_code(&email.parse().unwrap())
        .await
        .unwrap()
        .1;

    let verify_body = json!({
        "email": email,
        "loginAttemptId": login.login_attempt_id,
        "2FACode": code.as_ref()
    });

    let first_response = app.post_verify_2fa(&verify_body).await;
    assert_eq!(first_response.status().as_u16(), 200);

    let auth_cookie = first_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("no auth cookie found");
    assert!(!auth_cookie.value().is_empty());

    let second_response = app.post_verify_2fa(&verify_body).await;
    assert_eq!(second_response.status().as_u16(), 401);
}
