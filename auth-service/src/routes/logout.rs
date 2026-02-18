use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::{CookieJar, cookie::Cookie};

use crate::{
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = jar.get(JWT_COOKIE_NAME).ok_or(AuthAPIError::MissingToken)?;
    let token = cookie.value().to_string();

    // asserts the token is still valid
    let _ = validate_token(&token).await?;

    let updated_jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    Ok((updated_jar, StatusCode::OK))
}
