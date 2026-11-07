use std::sync::Arc;
use uuid::Uuid;
use serde::Serialize;

use crate::donations::models::NewDonation;
use crate::donations::repository::DonationRepository;
use crate::shared::errors::{AppError, AppResult};
use crate::organizations::repository::OrgRepository;
use crate::stripe::client::StripeClient;
use crate::stripe::models::{CreateStripeAccountRequest, CreateDonationRequest, EntityType};

#[derive(Clone)]
pub struct StripeService {
    pub client: Arc<StripeClient>,
    pub org_repo: Arc<OrgRepository>,
    pub donation_repo: Arc<DonationRepository>,
    pub frontend_base: String,
}

impl StripeService {
    pub fn new(
        client: Arc<StripeClient>,
        org_repo: Arc<OrgRepository>,
        donation_repo: Arc<DonationRepository>,
        frontend_base: String,
    ) -> Self {
        Self {
            client,
            org_repo,
            donation_repo,
            frontend_base,
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

        self.org_repo
            .update_stripe_status(org_id, charges_enabled, payouts_enabled, details_submitted)
            .await?;

        Ok(StripeOnboardingStatus {
            account_id,
            charges_enabled,
            payouts_enabled,
            onboarding_complete,
            requirements_currently_due: currently_due,
            requirements_eventually_due: eventually_due,
        })
    }

    /// Create a hosted Checkout Session for a donation. Public endpoint:
    /// the org is resolved by subdomain, no auth.
    pub async fn create_donation_checkout(
        &self,
        subdomain: &str,
        req: CreateDonationRequest,
    ) -> AppResult<(String, Uuid)> {
        let org = self.org_repo.find_by_subdomain(subdomain).await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        let account_id = org.stripe_account_id.clone()
            .ok_or_else(|| AppError::BadRequest("Organization Stripe account not set up".to_string()))?;

        if !org.stripe_complete() {
            return Err(AppError::BadRequest("Organization cannot accept payments yet".to_string()));
        }

        if req.amount_cents <= 0 {
            return Err(AppError::BadRequest("amount_cents must be positive".to_string()));
        }

        // shortcut: one silent campaign per org by construction (see OrgRepository::create)
        let campaign = self.org_repo.find_campaign_by_org(org.id).await?
            .ok_or_else(|| AppError::Internal)?;

        let donation_id = Uuid::new_v4();
        let platform_fee_cents = platform_fee(req.amount_cents, org.fee_bps);

        self.donation_repo.create(NewDonation {
            id: donation_id,
            campaign_id: campaign.id,
            amount_cents: req.amount_cents,
            currency: req.currency.clone(),
            status: "pending".to_string(),
            platform_fee_cents,
            donor_email: req.donor_email.clone(),
            donor_name: req.donor_name.clone(),
            metadata: req.metadata.clone(),
        }).await?;

        let success_url = format!(
            "{}/donate/{}/success?donation_id={}",
            self.frontend_base, subdomain, donation_id
        );
        let cancel_url = format!("{}/donate/{}/cancel", self.frontend_base, subdomain);

        let session = self.client.create_checkout_session(
            req.amount_cents,
            &req.currency,
            platform_fee_cents,
            &account_id,
            donation_id,
            org.id,
            &org.name,
            req.donor_email.as_deref(),
            &success_url,
            &cancel_url,
        ).await?;

        let checkout_url = session.url
            .ok_or_else(|| AppError::Internal)?;

        Ok((checkout_url, donation_id))
    }

    /// Public donation status lookup, scoped to the org subdomain.
    pub async fn donation_status(
        &self,
        subdomain: &str,
        donation_id: Uuid,
    ) -> AppResult<crate::stripe::models::DonationStatusResponse> {
        let donation = self.donation_repo
            .find_by_id_and_subdomain(donation_id, subdomain)
            .await?
            .ok_or_else(|| AppError::NotFound("Donation not found".to_string()))?;

        Ok(crate::stripe::models::DonationStatusResponse {
            donation_id: donation.id,
            status: donation.status,
            amount_cents: donation.amount_cents,
            platform_fee_cents: donation.platform_fee_cents.unwrap_or(0),
            currency: donation.currency,
        })
    }

    /// Verify callers handle dedupe; this stores every event and only acts on
    /// checkout session outcomes.
    pub async fn process_webhook_event(&self, event: &serde_json::Value) -> AppResult<()> {
        let event_id = event["id"].as_str()
            .ok_or_else(|| AppError::BadRequest("Missing event id".to_string()))?;
        let event_type = event["type"].as_str().unwrap_or("unknown");

        if !self.donation_repo.insert_event(event_id, event_type, event).await? {
            tracing::info!(event_id, "duplicate webhook event ignored");
            return Ok(());
        }

        let Some(status) = event_to_status(event_type) else {
            return Ok(());
        };

        let object = &event["data"]["object"];
        let Some(donation_id) = object["metadata"]["cream_donation_id"]
            .as_str()
            .and_then(|v| Uuid::parse_str(v).ok())
        else {
            tracing::warn!(event_id, event_type, "checkout event without cream_donation_id");
            return Ok(());
        };

        let payment_intent_id = object["payment_intent"].as_str();

        self.donation_repo
            .update_status(donation_id, status, payment_intent_id)
            .await
    }
}

/// Platform fee in cents, rounded down.
pub fn platform_fee(amount_cents: i64, fee_bps: i32) -> i64 {
    (amount_cents as i128 * fee_bps as i128 / 10_000) as i64
}

/// Checkout event type -> donation status. None = event not relevant to donations.
pub fn event_to_status(event_type: &str) -> Option<&'static str> {
    match event_type {
        "checkout.session.completed" | "checkout.session.async_payment_succeeded" => Some("succeeded"),
        "checkout.session.async_payment_failed" => Some("failed"),
        "checkout.session.expired" => Some("expired"),
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_fee_floors_to_cents() {
        assert_eq!(platform_fee(10_000, 500), 500); // 5% of $100
        assert_eq!(platform_fee(1_999, 500), 99); // floors 99.95 -> 99
        assert_eq!(platform_fee(100, 0), 0);
        assert_eq!(platform_fee(0, 500), 0);
    }

    #[test]
    fn webhook_event_mapping() {
        assert_eq!(event_to_status("checkout.session.completed"), Some("succeeded"));
        assert_eq!(event_to_status("checkout.session.async_payment_succeeded"), Some("succeeded"));
        assert_eq!(event_to_status("checkout.session.async_payment_failed"), Some("failed"));
        assert_eq!(event_to_status("checkout.session.expired"), Some("expired"));
        assert_eq!(event_to_status("customer.created"), None);
    }
}

