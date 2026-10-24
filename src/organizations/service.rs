use tracing::instrument;
use uuid::Uuid;

use crate::organizations::models::{CreateOrganizationRequest, NewCampaign, NewOrganization, Organization};
use crate::organizations::repository::OrgRepository;
use crate::shared::errors::{AppError, AppResult};

pub struct OrganizationService {
    org_repo: OrgRepository,
}

impl OrganizationService {
    pub fn new(org_repo: OrgRepository) -> Self {
        Self { org_repo }
    }

    #[instrument(skip(self, req), fields(subdomain = %req.subdomain))]
    pub async fn create(&self, req: CreateOrganizationRequest) -> AppResult<Organization> {
        let (name, subdomain) = validate(&req.name, &req.subdomain)?;

        let org = NewOrganization {
            id: Uuid::new_v4(),
            name,
            subdomain: subdomain.clone(),
            fee_bps: req.fee_bps,
            stripe_account_id: None,
        };

        // silent default campaign — donor sees only a form, no campaign abstraction
        let campaign = NewCampaign {
            id: Uuid::new_v4(),
            organization_id: org.id,
            slug: subdomain,
            title: org.name.clone(),
            description: None,
            currency: "USD".to_string(),
            active: Some(true),
        };

        let (org, _) = self.org_repo.create(org, campaign).await?;
        Ok(org)
    }
}

fn validate(name: &str, subdomain: &str) -> AppResult<(String, String)> {
    let name = name.trim().to_string();
    let subdomain = subdomain.trim().to_lowercase();
    if name.is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    if subdomain.is_empty() || !subdomain.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(AppError::BadRequest("subdomain must be [a-z0-9-]".to_string()));
    }
    Ok((name, subdomain))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_rejects_bad_subdomain() {
        assert!(validate("x", "Bad Sub").is_err());
        assert!(validate("x", "").is_err());
        assert_eq!(validate(" My-Org2 ", " My-Org2 ").ok().map(|(_, s)| s), Some("my-org2".to_string()));
    }
}
