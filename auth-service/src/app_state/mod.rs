use std::sync::Arc;

use crate::domain::{BannedTokenStore, EmailClient, TwoFACodeStore, UserStore};
use crate::services::{DashMapTwoFACodeStore, DashMapUserStore, DashSetBannedTokenStore};

#[derive(Clone)]
pub struct AppState {
    pub user_store: Arc<dyn UserStore + Send + Sync>,
    pub banned_token_store: Arc<dyn BannedTokenStore + Send + Sync>,
    pub two_fa_code_store: Arc<dyn TwoFACodeStore + Send + Sync>,
    pub email_client: Arc<dyn EmailClient + Send + Sync>,
}

impl AppState {
    pub fn new_tester(
        user_store: Arc<DashMapUserStore>,
        banned_token_store: Arc<DashSetBannedTokenStore>,
        two_fa_code_store: Arc<DashMapTwoFACodeStore>,
        email_client: Arc<dyn EmailClient + Send + Sync>,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }
}
