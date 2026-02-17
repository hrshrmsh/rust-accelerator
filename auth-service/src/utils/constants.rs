use std::sync::LazyLock;

use dotenvy::dotenv;

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
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
