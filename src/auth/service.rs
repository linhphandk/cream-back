use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::models::{NewOrganization, Organization};
use crate::auth::ports::OrganizationRepository;
use crate::shared::errors::{AppError, AppResult};

pub struct OrganizationService<R: OrganizationRepository> {
    org_repo: Arc<R>,
}

impl<R: OrganizationRepository> Clone for OrganizationService<R> {
    fn clone(&self) -> Self {
        Self {
            org_repo: Arc::clone(&self.org_repo),
        }
    }
}

impl<R: OrganizationRepository> OrganizationService<R> {
    pub fn new(org_repo: R) -> Self {
        Self {
            org_repo: Arc::new(org_repo),
        }
    }

    pub async fn create_organization(
        &self,
        name: &str,
        subdomain: &str,
        fee_bps: i32,
    ) -> AppResult<Organization> {
        if name.trim().is_empty() || subdomain.trim().is_empty() {
            return Err(AppError::Validation(
                "name and subdomain are required".to_string(),
            ));
        }

        if self.org_repo.find_by_subdomain(subdomain).await?.is_some() {
            return Err(AppError::Conflict(format!(
                "subdomain '{}' is already taken",
                subdomain
            )));
        }

        let new_org = NewOrganization {
            id: Uuid::new_v4(),
            name: name.to_string(),
            subdomain: subdomain.to_string(),
            fee_bps,
            stripe_account_id: None,
        };

        self.org_repo.create(new_org).await
    }

    pub async fn get_organization(&self, org_id: Uuid) -> AppResult<Organization> {
        self.org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn get_organization_by_subdomain(&self, subdomain: &str) -> AppResult<Organization> {
        self.org_repo
            .find_by_subdomain(subdomain)
            .await?
            .ok_or(AppError::NotFound)
    }
}

// re-export async_trait for consumers
pub use async_trait::async_trait as _async_trait_reexport;
