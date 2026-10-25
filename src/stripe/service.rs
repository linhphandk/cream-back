use std::sync::Arc;
use uuid::Uuid;
use serde::Serialize;

use crate::shared::errors::{AppError, AppResult};
use crate::organizations::models::Organization;
use crate::organizations::repository::OrgRepository;
use crate::stripe::client::StripeClient;
use crate::stripe::models::{CreateStripeAccountRequest, CreateDonationRequest, EntityType};

#[derive(Clone)]
pub struct StripeService {
    pub client: Arc<StripeClient>,
    pub org_repo: Arc<OrgRepository>,
    pub frontend_base: String,
    platform_fee_bps: i32,
}

impl StripeService {
    pub fn new(
        client: Arc<StripeClient>,
        org_repo: Arc<OrgRepository>,
        frontend_base: String,
        platform_fee_bps: i32,
    ) -> Self {
        Self {
            client,
            org_repo,
            frontend_base,
            platform_fee_bps,
        }
    }

    /// Org being onboarded/status-checked must belong to the caller.
    /// 404 if the org doesn't exist, 403 if the caller isn't a member.
    pub async fn assert_org_member(&self, org_id: Uuid, user_id: Uuid) -> AppResult<()> {
        if self.org_repo.find_by_id(org_id).await?.is_none() {
            return Err(AppError::NotFound("Organization not found".to_string()));
        }
        if self.org_repo.find_member(org_id, user_id).await?.is_none() {
            return Err(AppError::Forbidden("not a member of this organization".to_string()));
        }
        Ok(())
    }

    /// Create Stripe Connect account and onboarding session for an organization
    pub async fn create_onboarding(
        &self,
        org_id: Uuid,
        entity_type: EntityType,
        country: String,
    ) -> AppResult<(String, String)> {
        // Get org details
        let org = self.org_repo.find_by_id(org_id).await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // Create v2 account
        let req = CreateStripeAccountRequest {
            org_id,
            org_name: org.name.clone(),
            org_admin_email: "admin@example.com".to_string(), // TODO: get from user
            entity_type,
            country,
        };

        let account_resp = self.client.create_account_v2(&req, &self.frontend_base).await?;
        
        // Create account session for onboarding
        let session_resp = self.client.create_account_session(
            &account_resp.id,
            &self.frontend_base,
            org_id,
        ).await?;

        // Update org with stripe account ID
        self.org_repo.update_stripe_account_id(org_id, account_resp.id.clone()).await?;

        // Build onboarding URL
        let onboarding_url = session_resp.url;

        Ok((account_resp.id, onboarding_url))
    }

    /// Check Stripe account onboarding status
    pub async fn check_onboarding_status(&self, org_id: Uuid) -> AppResult<StripeOnboardingStatus> {
        let org = self.org_repo.find_by_id(org_id).await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        let account_id = org.stripe_account_id
            .ok_or_else(|| AppError::BadRequest("Stripe account not created yet".to_string()))?;

        let resp = self.client.retrieve_account(&account_id).await?;

        let charges_enabled = resp.charges_enabled;
        let payouts_enabled = resp.payouts_enabled;
        let details_submitted = resp.details_submitted;

        let requirements = resp.requirements.unwrap_or_default();
        let currently_due = requirements.currently_due;
        let eventually_due = requirements.eventually_due;

        let onboarding_complete = charges_enabled && payouts_enabled && details_submitted;

        // Update org status
        let status = if onboarding_complete {
            "active"
        } else if details_submitted {
            "pending"
        } else {
            "incomplete"
        };
        
        self.org_repo.update_stripe_status(org_id, charges_enabled, payouts_enabled, onboarding_complete, status).await?;

        Ok(StripeOnboardingStatus {
            account_id,
            charges_enabled,
            payouts_enabled,
            onboarding_complete,
            requirements_currently_due: currently_due,
            requirements_eventually_due: eventually_due,
        })
    }

    /// Create a PaymentIntent for a donation
    pub async fn create_donation_payment_intent(
        &self,
        subdomain: &str,
        req: CreateDonationRequest,
    ) -> AppResult<(String, Uuid, String, i64)> {
        // Get org by subdomain
        let org = self.org_repo.find_by_subdomain(subdomain).await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // Verify Stripe account is ready
        let account_id = org.stripe_account_id
            .ok_or_else(|| AppError::BadRequest("Organization Stripe account not set up".to_string()))?;

        if !org.stripe_charges_enabled.unwrap_or(false) {
            return Err(AppError::BadRequest("Organization cannot accept payments yet".to_string()));
        }

        let donation_id = Uuid::new_v4();
        let platform_fee_cents = (req.amount_cents as i128 * self.platform_fee_bps as i128 / 10000) as i64;

        let pi_resp = self.client.create_payment_intent(
            req.amount_cents,
            &req.currency,
            platform_fee_cents,
            &account_id,
            donation_id,
            org.id,
            req.donor_email.as_deref(),
            req.donor_name.as_deref(),
            req.metadata.as_ref(),
        ).await?;

        Ok((pi_resp.client_secret, donation_id, pi_resp.id, platform_fee_cents))
    }
}

#[derive(Debug, Serialize)]
pub struct StripeOnboardingStatus {
    pub account_id: String,
    pub charges_enabled: bool,
    pub payouts_enabled: bool,
    pub onboarding_complete: bool,
    pub requirements_currently_due: Vec<String>,
    pub requirements_eventually_due: Vec<String>,
}
