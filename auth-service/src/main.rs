use std::sync::Arc;

use auth_service::{
    Application,
    app_state::AppState,
    get_postgres_pool,
    services::{
        DashMapTwoFACodeStore, DashSetBannedTokenStore, MockEmailClient, PostgresUserStore,
    },
    utils::constants::{DATABASE_URL, prod},
};
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;

    let user_store = Arc::new(PostgresUserStore::new(pg_pool));
    let banned_token_store = Arc::new(DashSetBannedTokenStore::default());
    let two_fa_code_store = Arc::new(DashMapTwoFACodeStore::default());
    let email_client = Arc::new(MockEmailClient);
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
