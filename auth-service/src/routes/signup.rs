use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, User},
};

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    if email.len() < 1 || !email.contains('@') || password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User::new(email, password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;
    user_store.add_user(user)?;

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
