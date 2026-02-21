use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password},
    utils::auth,
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email: Email = request.email.parse()?;
    let password: Password = request.password.parse()?;

    state.user_store.validate_user(&email, &password).await?;

    match state.user_store.get_user(&email).await?.requires_2fa {
        true => Ok((
            jar,
            (StatusCode::PARTIAL_CONTENT, Json(LoginResponse::new())),
        )
            .into_response()),
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
    pub login_attemp_id: String,
}

impl LoginResponse {
    fn new() -> Self {
        Self {
            message: String::from("2fa required!"),
            login_attemp_id: String::from("123456"), // TODO: add login attempt ids
        }
    }
}
