use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::{
    UserState,
    domain::{AuthAPIError, Email, Password, UserStore},
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
