use std::sync::LazyLock;

use dotenvy::dotenv;
use secrecy::SecretString;

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DROPLET_IP_ENV_VAR: &str = "DROPLET_IP";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOSTNAME";
    pub const POSTMARK_AUTH_TOKEN_ENV_VAR: &str = "POSTMARK_AUTH_TOKEN";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
    pub mod email_client {
        use std::time::Duration;

        pub const BASE_URL: &str = "https://api.postmarkapp.com/email";
        pub const SENDER: &str = "bogdan@codeiron.io";
        pub const TIMEOUT: Duration = std::time::Duration::from_secs(10);
    }
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
    pub mod email_client {
        use std::time::Duration;

        pub const SENDER: &str = "test@email.com";
        pub const TIMEOUT: Duration = std::time::Duration::from_millis(200);
    }
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

// load env vars every lock for compatability with multiple env files (i.e. test env)
// overhead is fairly minimal for a web server
pub static JWT_SECRET: LazyLock<SecretString> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std::env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set!");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty!");
    }
    secret.into()
});

pub static DROPLET_IP: LazyLock<SecretString> = LazyLock::new(|| {
    dotenv().ok();
    let ip = std::env::var(env::DROPLET_IP_ENV_VAR).expect("DROPLET_IP must be set!");
    if ip.is_empty() {
        panic!("DROPLET_IP must not be empty!");
    }
    ip.into()
});

pub static DATABASE_URL: LazyLock<SecretString> = LazyLock::new(|| {
    dotenv().ok();
    let db_url = std::env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set!");
    if db_url.is_empty() {
        panic!("DATABASE_URL must not be empty!");
    }
    db_url.into()
});

pub static REDIS_HOST_NAME: LazyLock<SecretString> = LazyLock::new(|| {
    dotenv().ok();
    std::env::var(env::REDIS_HOST_NAME_ENV_VAR)
        .unwrap_or(DEFAULT_REDIS_HOSTNAME.to_string())
        .into()
});

pub static POSTMARK_AUTH_TOKEN: LazyLock<SecretString> = LazyLock::new(|| {
    dotenv().ok();
    let token =
        std::env::var(env::POSTMARK_AUTH_TOKEN_ENV_VAR).expect("POSTMARK_AUTH_TOKEN must be set!");
    if token.is_empty() {
        panic!("POSTMARK_AUTH_TOKEN must not be empty!");
    }
    token.into()
});
