use std::error::Error;

use axum::{Router, serve::Serve};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        let assets_dir = ServeDir::new("assets");
        let router = Router::new().fallback_service(assets_dir);

        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self {
            server,
            address,
        })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("server started! listening on {}.", &self.address);
        self.server.await
    }
}