use std::time::Duration;

use axum::{body::Body, http::Request, response::Response};
use color_eyre::eyre::Result;
use tracing::{Level, Span, event, field, span};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use uuid::Uuid;

pub fn init_tracing() -> Result<()> {
    let fmt_layer = fmt::layer().compact();
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

// creates a new tracing span with a unique request id for each incoming request.
// helps in tracking & correlating logs for individual requests
pub fn make_span_with_request_id(request: &Request<Body>) -> Span {
    let request_id = Uuid::new_v4();
    span!(
        Level::INFO,
        "[REQUEST]",
        method = field::display(request.method()),
        uri = field::display(request.uri()),
        version = field::debug(request.version()),
        request_id = field::display(request_id),
    )
}

// logs on event indicating the start of a request
pub fn on_request(_: &Request<Body>, _: &Span) {
    event!(Level::INFO, "[REQUEST START]");
}

// logs an event indicating the end of a request, including latency & status code
// if status code is error (4XX/5XX), logs at ERROR level
pub fn on_response(response: &Response, latency: Duration, _: &Span) {
    let status = response.status();
    let status_code = status.as_u16();

    if status.is_server_error() || status.is_client_error() {
        event!(Level::ERROR, latency = ?latency, status = status_code, "[REQUEST END]")
    } else {
        event!(Level::INFO, latency = ?latency, status = status_code, "[REQUEST END]")
    }
}
