use std::sync::Arc;

use auth_service::{
    Application,
    app_state::AppState,
    domain::Email,
    get_postgres_pool, get_redis_client,
    services::{
        PostgresUserStore, PostmarkEmailClient, RedisBannedTokenStore, RedisTwoFACodeStore,
    },
    utils::{
        constants::{DATABASE_URL, POSTMARK_AUTH_TOKEN, REDIS_HOST_NAME, prod},
        tracing::init_tracing,
    },
};
use reqwest::Client;
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("failed to install color_eyre");
    init_tracing().expect("failed to initialize tracing");

    let pg_pool = configure_postgresql().await;
    let redis_connection = Arc::new(RwLock::new(configure_redis().await));

    let user_store = Arc::new(PostgresUserStore::new(pg_pool));
    let banned_token_store = Arc::new(RedisBannedTokenStore::new(redis_connection.clone()));
    let two_fa_code_store = Arc::new(RedisTwoFACodeStore::new(redis_connection));
    let email_client = Arc::new(configure_postmark_email_client());
    let app_state = AppState {
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    };

    // Here we are using ip 0.0.0.0 so the service is listening on all the configured network interfaces.
    // This is needed for Docker to work, which we will add later on.
    // See: https://stackoverflow.com/questions/39525820/docker-port-forwarding-not-working
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("failed to build app!");

    app.run().await.expect("app crashed trying to run!");
}

async fn configure_postgresql() -> PgPool {
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("failed to create postgres connection pool!");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("failed migrations!");

    pg_pool
}

async fn configure_redis() -> redis::aio::MultiplexedConnection {
    get_redis_client(&REDIS_HOST_NAME)
        .expect("failed to get redis client")
        .get_multiplexed_async_connection()
        .await
        .expect("failed to get redis connection")
}

fn configure_postmark_email_client() -> PostmarkEmailClient {
    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("failed to build http client");

    PostmarkEmailClient::new(
        http_client,
        prod::email_client::BASE_URL.into(),
        Email::parse(prod::email_client::SENDER.into()).expect("invalid email client"),
        POSTMARK_AUTH_TOKEN.clone(),
    )
}
