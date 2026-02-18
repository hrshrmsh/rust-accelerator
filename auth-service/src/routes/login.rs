use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password},
    utils::auth,
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email: Email = request.email.parse()?;
    let password: Password = request.password.parse()?;

    state.user_store.validate_user(&email, &password).await?;

    let auth_cookie = auth::generate_auth_cookie(&email)?;
    let new_jar = jar.add(auth_cookie);

    Ok((new_jar, StatusCode::OK.into_response()))
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
