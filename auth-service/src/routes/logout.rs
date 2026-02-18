use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{CookieJar, cookie::Cookie};

use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = jar.get(JWT_COOKIE_NAME).ok_or(AuthAPIError::MissingToken)?;
    let token = cookie.value().to_string();

    // asserts the token is still valid
    let _ = validate_token(state.banned_token_store.clone(), &token).await?;

    let updated_jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));
    state.banned_token_store.add_token(&token).await?;

    Ok((updated_jar, StatusCode::OK))
}
