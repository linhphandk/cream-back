use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::models::{NewOrganization, Organization, UserInfo};
use crate::shared::errors::AppResult;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn create(&self, org: NewOrganization) -> AppResult<Organization>;
    async fn find_by_subdomain(&self, subdomain: &str) -> AppResult<Option<Organization>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Organization>>;
    async fn update_stripe_account_id(&self, id: Uuid, stripe_account_id: String) -> AppResult<()>;
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn hash_password(&self, password: &str) -> AppResult<String>;
    async fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
    async fn introspect_token(&self, token: &str) -> AppResult<UserInfo>;
    fn token_expiry_seconds(&self) -> u64;
}