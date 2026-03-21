use std::error::Error;

use axum::{Router, http::Method, routing::post, serve::Serve};
use redis::{Client, RedisResult};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

use app_state::AppState;
use tracing::info;

use crate::utils::{
    constants::DROPLET_IP,
    tracing::{make_span_with_request_id, on_request, on_response},
};

pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let assets_dir =
            ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));

        let allowed_origins = [
            "http://localhost:8000".parse()?,
            format!("http://{}:8000", *DROPLET_IP).parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .fallback_service(assets_dir)
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/logout", post(routes::logout))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::verify_token))
            .with_state(app_state)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span_with_request_id)
                    .on_request(on_request)
                    .on_response(on_response),
            );

        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        info!("server started! listening on {}", &self.address);
        self.server.await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().max_connections(5).connect(url).await
}

pub fn get_redis_client(redis_hostname: String) -> RedisResult<Client> {
    let url = format!("redis://{}/", redis_hostname);
    redis::Client::open(url)
}
