use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{AuthAPIError, User, UserStore},
    UserState,
};

pub async fn signup(
    State(state): UserState,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    let user = User::new(email.parse()?, password.parse()?, request.requires_2fa);

    let mut user_store = state.user_store.write().await;
    user_store.add_user(user).await?;

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
