use std::sync::Arc;

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
    pub async fn create_organization(&self, req: CreateOrganizationRequest) -> AppResult<Organization> {
        // user identity will be threaded in here (owner) once auth middleware lands
        self.org_service.create(req).await
    }
}
