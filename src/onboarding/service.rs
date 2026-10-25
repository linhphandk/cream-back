use std::sync::Arc;

use uuid::Uuid;

use crate::organizations::models::{CreateOrganizationRequest, Organization};
use crate::organizations::service::OrganizationService;
use crate::shared::errors::AppResult;

/// Orchestrates org provisioning steps. Register (auth) → create_organization → stripe (M2).
#[derive(Clone)]
pub struct OnboardingService {
    org_service: Arc<OrganizationService>,
}

impl OnboardingService {
    pub fn new(org_service: Arc<OrganizationService>) -> Self {
        Self { org_service }
    }

    /// Step: create org (+ silent default campaign) for an onboarded user.
    pub async fn create_organization(&self, req: CreateOrganizationRequest, owner_id: Uuid) -> AppResult<Organization> {
        self.org_service.create(req, owner_id).await
    }

    /// The caller's org, if any (one org per user is enforced at the DB level).
    pub async fn my_org(&self, user_id: Uuid) -> AppResult<Option<Organization>> {
        self.org_service.find_by_member(user_id).await
    }

    /// Public org info for the donate page.
    pub async fn org_by_subdomain(&self, sub: &str) -> AppResult<Option<Organization>> {
        self.org_service.find_by_subdomain(sub).await
    }
}
