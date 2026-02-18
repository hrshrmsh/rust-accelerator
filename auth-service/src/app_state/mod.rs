use std::sync::Arc;

use crate::domain::{BannedTokenStore, UserStore};
use crate::services::{HashMapUserStore, HashSetTokenStore};

#[derive(Clone)]
pub struct AppState {
    pub user_store: Arc<dyn UserStore + Send + Sync>,
    pub banned_token_store: Arc<dyn BannedTokenStore + Send + Sync>,
}

impl AppState {
    // test impl
    pub fn new_tester(
        user_store: Arc<HashMapUserStore>,
        banned_token_store: Arc<HashSetTokenStore>,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
        }
    }

    // generic impl (for prod)
    pub fn new(
        user_store: impl UserStore + Send + Sync + 'static,
        banned_token_store: impl BannedTokenStore + Send + Sync + 'static,
    ) -> Self {
        Self {
            user_store: Arc::new(user_store),
            banned_token_store: Arc::new(banned_token_store),
        }
    }
}
