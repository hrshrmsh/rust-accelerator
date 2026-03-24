use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use tracing::instrument;

use crate::{app_state::AppState, domain::AuthAPIError, utils::auth::validate_token};

#[instrument(name = "Verify Token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let _ = validate_token(state.banned_token_store, &request.token).await?;

    Ok(StatusCode::OK.into_response())
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
