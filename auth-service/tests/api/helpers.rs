use auth_service::domain::Email;
use auth_service::get_redis_client;
use auth_service::services::{PostmarkEmailClient, RedisBannedTokenStore, RedisTwoFACodeStore};
use auth_service::utils::constants::REDIS_HOST_NAME;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use sqlx::Connection;
use std::str::FromStr;
use std::{fmt::Debug, sync::Arc};
use test_context::AsyncTestContext;
use tokio::sync::RwLock;
use wiremock::MockServer;

use auth_service::{
    Application,
    app_state::AppState,
    get_postgres_pool,
    services::PostgresUserStore,
    utils::constants::{DATABASE_URL, test},
};

use reqwest::cookie::Jar;
use sqlx::{
    PgConnection, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_name: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: Arc<RedisBannedTokenStore>,
    pub two_fa_code_store: Arc<RedisTwoFACodeStore>,
    pub email_server: MockServer,
    pub ready_to_drop: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (db_name, pg_pool) = configure_postgresql().await;
        let redis_connection = Arc::new(RwLock::new(configure_redis().await));
        let email_server = MockServer::start().await;

        let user_store = Arc::new(PostgresUserStore::new(pg_pool));
        let banned_token_store = Arc::new(RedisBannedTokenStore::new(redis_connection.clone()));
        let two_fa_code_store = Arc::new(RedisTwoFACodeStore::new(redis_connection));
        let base_url = email_server.uri();
        let email_client = Arc::new(configure_postmark_email_client(base_url));

        let app_state = AppState {
            user_store: user_store.clone(),
            banned_token_store: banned_token_store.clone(),
            two_fa_code_store: two_fa_code_store.clone(),
            email_client: email_client.clone(),
        };

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
            db_name,
            cookie_jar,
            http_client,
            email_server,
            banned_token_store,
            two_fa_code_store,
            ready_to_drop: false,
        }
    }

    pub async fn clean_up(&mut self) {
        if self.ready_to_drop {
            panic!("cannot clean app twice!")
        }
        delete_database(&self.db_name).await;
        self.ready_to_drop = true;
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

impl AsyncTestContext for TestApp {
    async fn setup() -> Self {
        TestApp::new().await
    }

    async fn teardown(mut self) {
        self.clean_up().await;
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.ready_to_drop {
            panic!("dropped test app before cleaning up!")
        }
    }
}

async fn configure_postgresql() -> (String, PgPool) {
    let postgresql_conn_url = DATABASE_URL.expose_secret();
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    (
        db_name,
        get_postgres_pool(&postgresql_conn_url_with_db.into())
            .await
            .expect("Failed to create Postgres connection pool!"),
    )
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
        .execute(&connection)
        .await
        .expect("Failed to create database.");

    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.expose_secret().to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::query(&format!(
        r#"
            SELECT pg_terminate_backend(pg_stat_activity.pid)
            FROM pg_stat_activity
            WHERE pg_stat_activity.datname = '{}'
                AND pid <> pg_backend_pid();"#,
        db_name
    ))
    .execute(&mut connection)
    .await
    .expect("Failed to drop the database.");

    // Drop the database
    sqlx::query(&format!(r#"DROP DATABASE "{}";"#, db_name))
        .execute(&mut connection)
        .await
        .expect("Failed to drop the database.");
}

async fn configure_redis() -> redis::aio::MultiplexedConnection {
    get_redis_client(&REDIS_HOST_NAME)
        .expect("failed to get redis client")
        .get_multiplexed_async_connection()
        .await
        .expect("failed to get redis connection")
}

fn configure_postmark_email_client(base_url: String) -> PostmarkEmailClient {
    let postmark_auth_token: SecretString = "auth_token".into();

    let sender = Email::parse(SecretString::new(
        test::email_client::SENDER.to_owned().into_boxed_str(),
    ))
    .unwrap();

    let http_client = Client::builder()
        .timeout(test::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(http_client, base_url.into(), sender, postmark_auth_token)
}
