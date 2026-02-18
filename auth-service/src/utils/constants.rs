use std::sync::LazyLock;

use dotenvy::dotenv;

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DROPLET_IP_ENV_VAR: &str = "DROPLET_IP";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

pub const JWT_COOKIE_NAME: &str = "jwt";

pub static JWT_SECRET: LazyLock<String> = LazyLock::new(|| {
    // load env vars
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
