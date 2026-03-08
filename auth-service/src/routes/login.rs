use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, HashedPassword, LoginAttemptId, TwoFACode},
    utils::auth,
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email: Email = request.email.parse()?;
    HashedPassword::parse(request.password.clone()).await?;

    state
        .user_store
        .validate_user(&email, &request.password)
        .await?;

    match state.user_store.get_user(&email).await?.requires_2fa {
        true => {
            let login_attempt_id = LoginAttemptId::default();
            let two_fa_code = TwoFACode::default();

            state
                .two_fa_code_store
                .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
                .await?;

            state
                .email_client
                .send_email(
                    &email,
                    "2FA required",
                    &format!(
                        "Hello user,\n\
                    Your 2FA code is: {}. \
                    Please enter it within the next 10 minutes.\n",
                        two_fa_code.as_ref()
                    ),
                )
                .await?;

            // this is a bit scuffed, but it seems to work well with axum
            Ok((
                jar,
                (
                    StatusCode::PARTIAL_CONTENT,
                    Json(LoginResponse::new(&login_attempt_id)),
                ),
            )
                .into_response())
        }
        false => {
            let auth_cookie = auth::generate_auth_cookie(&email)?;
            let new_jar = jar.add(auth_cookie);

            Ok((new_jar, (StatusCode::OK, ())).into_response())
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LoginResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

impl LoginResponse {
    fn new(login_attempt_id: &LoginAttemptId) -> Self {
        Self {
            message: String::from("2fa required!"),
            login_attempt_id: login_attempt_id.as_ref().to_string(),
        }
    }
}
