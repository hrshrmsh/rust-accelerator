use std::time::Duration;

use axum::{body::Body, http::Request, response::Response};
use tracing::{Level, Span, event, field, span};
use uuid::Uuid;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(Level::DEBUG)
        .init();
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
