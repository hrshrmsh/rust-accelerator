use auth_service::{
    Application,
    app_state::AppState,
    services::{DashMapTwoFACodeStore, DashMapUserStore, DashSetBannedTokenStore, MockEmailClient},
    utils::constants::prod,
};

#[tokio::main]
async fn main() {
    let user_store = DashMapUserStore::default();
    let banned_token_store = DashSetBannedTokenStore::default();
    let two_fa_code_store = DashMapTwoFACodeStore::default();
    let email_client = MockEmailClient;
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    // Here we are using ip 0.0.0.0 so the service is listening on all the configured network interfaces.
    // This is needed for Docker to work, which we will add later on.
    // See: https://stackoverflow.com/questions/39525820/docker-port-forwarding-not-working
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("failed to build app!");

    app.run().await.expect("app crashed trying to run!");
}
