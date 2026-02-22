use std::{fmt::Debug, sync::Arc};

use auth_service::{
    Application,
    app_state::AppState,
    services::{DashMapTwoFACodeStore, DashMapUserStore, DashSetBannedTokenStore, MockEmailClient},
    utils::constants::test,
};

use reqwest::cookie::Jar;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: Arc<DashSetBannedTokenStore>,
    pub two_fa_code_store: Arc<DashMapTwoFACodeStore>,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(DashMapUserStore::default());
        let banned_token_store = Arc::new(DashSetBannedTokenStore::default());
        let two_fa_code_store = Arc::new(DashMapTwoFACodeStore::default());
        let email_client = Arc::new(MockEmailClient);

        let app_state = AppState::new_tester(
            user_store.clone(),
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );

        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("could not build application");
        let address = format!("http://{}", &app.address);

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
        }
    }

    #[inline]
    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("could not execute request")
    }

    #[inline]
    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", self.address))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    #[inline]
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", self.address))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    #[inline]
    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", self.address))
            .send()
            .await
            .expect("failed to execute request")
    }

    #[inline]
    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize + Debug,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    #[inline]
    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    // temp helper fn
    pub fn get_random_email() -> String {
        format!("{}@example.com", Uuid::new_v4())
    }
}
