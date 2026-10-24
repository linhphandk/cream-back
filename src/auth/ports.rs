use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::models::{Tokens, UserInfo};
use crate::shared::errors::AppResult;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn hash_password(&self, password: &str) -> AppResult<String>;
    async fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
    async fn issue_tokens(&self, user: &UserInfo) -> AppResult<Tokens>;
    async fn introspect_token(&self, token: &str) -> AppResult<UserInfo>;
    fn token_expiry_seconds(&self) -> u64;
    fn jwt_secret(&self) -> &str;
}

pub struct LoginFlow {
    pub provider_id: Option<Uuid>,
}
