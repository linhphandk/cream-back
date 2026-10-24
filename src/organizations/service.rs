use tracing::instrument;
use uuid::Uuid;

use crate::organizations::models::{CreateOrganizationRequest, NewCampaign, NewOrganization, Organization, Campaign};
use crate::organizations::ports::OrganizationRepository;
use crate::shared::errors::{AppError, AppResult};

pub struct OrganizationService<R: OrganizationRepository> {
    org_repo: R,
}

impl<R: OrganizationRepository> OrganizationService<R> {
    pub fn new(org_repo: R) -> Self {
        Self { org_repo }
    }

    #[instrument(skip(self, req), fields(subdomain = %req.subdomain))]
    pub async fn create(&self, req: CreateOrganizationRequest) -> AppResult<Organization> {
        let req = validate_org(req)?;

        let new_org = NewOrganization {
            id: Uuid::new_v4(),
            name: req.name,
            subdomain: req.subdomain.clone(),
            fee_bps: req.fee_bps,
            stripe_account_id: None,
        };

        // silent default campaign — donor sees only a form, no campaign abstraction
        let new_campaign = NewCampaign {
            id: Uuid::new_v4(),
            organization_id: new_org.id,
            slug: req.subdomain,
            title: new_org.name.clone(),
            description: None,
            currency: "USD".to_string(),
            active: Some(true),
        };

        let (org, _) = self.org_repo.create(new_org, new_campaign).await?;
        Ok(org)
    }

    pub async fn by_subdomain(&self, subdomain: &str) -> AppResult<Organization> {
        self.org_repo
            .find_by_subdomain(subdomain)
            .await?
            .ok_or_else(|| AppError::NotFound("organization not found".to_string()))
    }

    pub async fn get_campaign_for_org(&self, org_id: Uuid) -> AppResult<Campaign> {
        self.org_repo
            .find_campaign_for_org(org_id)
            .await?
            .ok_or_else(|| AppError::NotFound("campaign not found".to_string()))
    }
}

fn validate_org(req: CreateOrganizationRequest) -> AppResult<CreateOrganizationRequest> {
    let name = req.name.trim().to_string();
    let subdomain = req.subdomain.trim().to_lowercase();

    if name.is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    if !subdomain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
        || subdomain.is_empty()
    {
        return Err(AppError::BadRequest(
            "subdomain must be [a-z0-9-]".to_string(),
        ));
    }
    Ok(CreateOrganizationRequest {
        name,
        subdomain,
        fee_bps: req.fee_bps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct NoopRepo;

    #[async_trait]
    impl OrganizationRepository for NoopRepo {
        async fn create(&self, _o: NewOrganization, _c: NewCampaign) -> AppResult<(Organization, Campaign)> {
            unimplemented!()
        }
        async fn find_by_subdomain(&self, _s: &str) -> AppResult<Option<Organization>> { unimplemented!() }
        async fn find_by_id(&self, _i: Uuid) -> AppResult<Option<Organization>> { unimplemented!() }
        async fn find_campaign_for_org(&self, _o: Uuid) -> AppResult<Option<Campaign>> { unimplemented!() }
        async fn update_stripe_account_id(&self, _i: Uuid, _a: String) -> AppResult<()> { unimplemented!() }
    }

    #[test]
    fn validation_rejects_bad_subdomain() {
        let svc = OrganizationService::new(NoopRepo);
        assert!(validate_org(CreateOrganizationRequest { name: "x".into(), subdomain: "Bad Sub".into(), fee_bps: 500 }).is_err());
        let ok = validate_org(CreateOrganizationRequest { name: "x".into(), subdomain: "  My-Org2 ".into(), fee_bps: 500 }).unwrap();
        assert_eq!(ok.subdomain, "my-org2");
        assert_eq!(ok.fee_bps, 500);
    }
}
