use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, HashedPassword, User},
};

#[instrument(name = "Signup", skip_all, err(Debug))]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    let user = User::new(
        email.parse()?,
        HashedPassword::parse(password).await?,
        request.requires_2fa,
    );

    state.user_store.add_user(user).await?;

    let response = Json(SignupResponse {
        message: String::from("User created successfully!"),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SignupResponse {
    pub message: String,
}
