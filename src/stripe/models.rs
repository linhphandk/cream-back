use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to create a Stripe Connect account for an organization
#[derive(Debug, Deserialize)]
pub struct CreateStripeAccountRequest {
    pub org_id: Uuid,
    pub org_name: String,
    pub org_admin_email: String,
    pub entity_type: EntityType,
    pub country: String,
}

/// Entity type for the connected account
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Company,
    Individual,
}

impl EntityType {
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::Company => "company",
            EntityType::Individual => "individual",
        }
    }
}

/// Response from creating a Stripe account
#[derive(Debug, Serialize)]
pub struct CreateStripeAccountResponse {
    pub account_id: String,
    pub client_secret: String,
    pub onboarding_url: String,
}

/// Stripe account status for polling
#[derive(Debug, Serialize)]
pub struct StripeAccountStatus {
    pub account_id: String,
    pub charges_enabled: bool,
    pub payouts_enabled: bool,
    pub onboarding_complete: bool,
    pub requirements_currently_due: Vec<String>,
    pub requirements_eventually_due: Vec<String>,
}

/// Request to create a donation PaymentIntent
#[derive(Debug, Deserialize)]
pub struct CreateDonationRequest {
    pub amount_cents: i64,
    pub currency: String,
    pub donor_email: Option<String>,
    pub donor_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Response from creating a donation PaymentIntent
#[derive(Debug, Serialize)]
pub struct CreateDonationResponse {
    pub client_secret: String,
    pub donation_id: Uuid,
    pub payment_intent_id: String,
}

/// Donation status response
#[derive(Debug, Serialize)]
pub struct DonationStatusResponse {
    pub donation_id: Uuid,
    pub status: String,
    pub amount_cents: i64,
    pub platform_fee_cents: i64,
    pub currency: String,
}
