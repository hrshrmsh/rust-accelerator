use std::sync::Arc;

use crate::domain::{BannedTokenStore, TwoFACodeStore, UserStore};
use crate::services::{DashMapTwoFACodeStore, DashMapUserStore, DashSetBannedTokenStore};

#[derive(Clone)]
pub struct AppState {
    pub user_store: Arc<dyn UserStore + Send + Sync>,
    pub banned_token_store: Arc<dyn BannedTokenStore + Send + Sync>,
    pub two_fa_code_store: Arc<dyn TwoFACodeStore + Send + Sync>,
}

impl AppState {
    // test impl
    pub fn new_tester(
        user_store: Arc<DashMapUserStore>,
        banned_token_store: Arc<DashSetBannedTokenStore>,
        two_fa_code_store: Arc<DashMapTwoFACodeStore>,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
        }
    }

    // generic impl (for prod)
    pub fn new(
        user_store: impl UserStore + Send + Sync + 'static,
        banned_token_store: impl BannedTokenStore + Send + Sync + 'static,
        two_fa_code_store: impl TwoFACodeStore + Send + Sync + 'static,
    ) -> Self {
        Self {
            user_store: Arc::new(user_store),
            banned_token_store: Arc::new(banned_token_store),
            two_fa_code_store: Arc::new(two_fa_code_store),
        }
    }
}
