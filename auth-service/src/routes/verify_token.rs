use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::{domain::AuthAPIError, utils::auth::validate_token};

pub async fn verify_token(
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let _ = validate_token(&request.token).await?;

    Ok(StatusCode::OK.into_response())
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
