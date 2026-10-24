use async_trait::async_trait;
use uuid::Uuid;

use crate::organizations::models::{Campaign, NewCampaign, NewOrganization, Organization};
use crate::shared::errors::AppResult;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn create(&self, org: NewOrganization, default_campaign: NewCampaign) -> AppResult<(Organization, Campaign)>;
    async fn find_by_subdomain(&self, subdomain: &str) -> AppResult<Option<Organization>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Organization>>;
    async fn find_campaign_for_org(&self, org_id: Uuid) -> AppResult<Option<Campaign>>;
    async fn update_stripe_account_id(&self, id: Uuid, stripe_account_id: String) -> AppResult<()>;
}
