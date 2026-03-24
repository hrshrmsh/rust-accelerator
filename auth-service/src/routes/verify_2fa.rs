use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use tracing::instrument;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
    utils::auth,
};

#[instrument(name = "Verify 2FA", skip_all)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let login_attempted_id = LoginAttemptId::parse(request.login_attempt_id)?;
    let two_fa_code = TwoFACode::parse(request.two_fa_code)?;

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

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    email: SecretString,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: SecretString,
    #[serde(rename = "2FACode")]
    two_fa_code: SecretString,
}

impl PartialEq for Verify2FARequest {
    fn eq(&self, other: &Self) -> bool {
        self.email.expose_secret() == other.email.expose_secret()
            && self.login_attempt_id.expose_secret() == other.login_attempt_id.expose_secret()
            && self.two_fa_code.expose_secret() == other.two_fa_code.expose_secret()
    }
}
