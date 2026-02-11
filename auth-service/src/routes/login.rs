use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{
    domain::{AuthAPIError, Email, Password, UserStore},
    UserState,
};

pub async fn login(
    State(state): UserState,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email: Email = request.email.parse()?;
    let password: Password = request.password.parse()?;

    state
        .user_store
        .read()
        .await
        .validate_user(&email, &password)
        .await?;

    Ok(StatusCode::OK.into_response())
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
