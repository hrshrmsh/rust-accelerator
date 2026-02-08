use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{domain::UserStore, services::HashmapUserStore};

pub type UserStoreType<T> = Arc<RwLock<T>>;

#[derive(Clone)]
pub struct AppState<T: UserStore> {
    pub user_store: UserStoreType<T>,
}

impl AppState<HashmapUserStore> {
    pub fn new(user_store: UserStoreType<HashmapUserStore>) -> Self {
        Self { user_store }
    }
}
