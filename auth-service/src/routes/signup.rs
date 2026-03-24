use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, HashedPassword, User},
};

#[instrument(name = "Signup", skip_all)]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    let user = User::new(
        Email::parse(email).map_err(|_| AuthAPIError::InvalidCredentials)?,
        HashedPassword::parse(password)
            .await
            .map_err(|_| AuthAPIError::InvalidCredentials)?,
        request.requires_2fa,
    );

    state.user_store.add_user(user).await?;

    let response = Json(SignupResponse {
        message: String::from("User created successfully!"),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: SecretString,
    pub password: SecretString,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

impl PartialEq for SignupRequest {
    fn eq(&self, other: &Self) -> bool {
        self.email.expose_secret() == other.email.expose_secret()
            && self.password.expose_secret() == other.password.expose_secret()
            && self.requires_2fa == other.requires_2fa
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SignupResponse {
    pub message: String,
}
