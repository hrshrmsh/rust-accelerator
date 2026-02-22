use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
    utils::auth,
};

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email: Email = request.email.parse()?;
    let login_attempted_id: LoginAttemptId = request.login_attempt_id.parse()?;
    let two_fa_code: TwoFACode = request.two_fa_code.parse()?;

    let (id, twofa) = state
        .two_fa_code_store
        .get_code(&email)
        .await
        .map_err(|_| AuthAPIError::AuthenticationError)?;

    if login_attempted_id != id || twofa != two_fa_code {
        return Err(AuthAPIError::AuthenticationError);
    }

    let auth_cookie = auth::generate_auth_cookie(&email)?;
    let new_jar = jar.add(auth_cookie);

    state.two_fa_code_store.remove_code(&email).await?;

    Ok((new_jar, StatusCode::OK).into_response())
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Verify2FARequest {
    email: String,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: String,
    #[serde(rename = "2FACode")]
    two_fa_code: String,
}
