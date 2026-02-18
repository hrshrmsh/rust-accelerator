use auth_service::{
    Application,
    app_state::AppState,
    services::{HashMapUserStore, HashSetTokenStore},
    utils::constants::prod,
};

#[tokio::main]
async fn main() {
    let user_store = HashMapUserStore::default();
    let banned_token_store = HashSetTokenStore::default();
    let app_state = AppState::new(user_store, banned_token_store);
    // Here we are using ip 0.0.0.0 so the service is listening on all the configured network interfaces.
    // This is needed for Docker to work, which we will add later on.
    // See: https://stackoverflow.com/questions/39525820/docker-port-forwarding-not-working
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("failed to build app!");

    app.run().await.expect("app crashed trying to run!");
}
