use std::sync::LazyLock;

use dotenvy::dotenv;

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DROPLET_IP_ENV_VAR: &str = "DROPLET_IP";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

pub const JWT_COOKIE_NAME: &str = "jwt";

// load env vars every lock for compatability with multiple env files (i.e. test env)
// overhead is fairly minimal for a web server
pub static JWT_SECRET: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let secret = std::env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set!");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty!");
    }
    secret
});

pub static DROPLET_IP: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let ip = std::env::var(env::DROPLET_IP_ENV_VAR).expect("DROPLET_IP must be set!");
    if ip.is_empty() {
        panic!("DROPLET_IP must not be empty!");
    }
    ip
});

pub static DATABASE_URL: LazyLock<String> = LazyLock::new(|| {
    dotenv().ok();
    let db_url = std::env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set!");
    if db_url.is_empty() {
        panic!("DATABASE_URL must not be empty!");
    }
    db_url
});
