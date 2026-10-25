use axum::{
    extract::{State, Path, Json, Query},
    http::{StatusCode, HeaderMap},
    response::{Redirect, IntoResponse},
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::stripe::service::StripeService;
use crate::shared::errors::{AppError, AppResult};
use crate::auth::extractor::CurrentUser;
use crate::stripe::models::{CreateStripeAccountRequest, CreateDonationRequest, EntityType};

pub type StripeState = std::sync::Arc<StripeService>;

#[derive(Deserialize)]
pub struct CreateOnboardingRequest {
    pub org_id: Uuid,
    pub entity_type: EntityType,
    #[serde(default = "default_country")]
    pub country: String,
}

fn default_country() -> String {
    "US".to_string()
}

#[derive(Serialize)]
pub struct CreateOnboardingResponse {
    pub account_id: String,
    pub onboarding_url: String,
}

#[derive(Serialize)]
pub struct OnboardingStatusResponse {
    pub account_id: String,
    pub charges_enabled: bool,
    pub payouts_enabled: bool,
    pub onboarding_complete: bool,
    pub requirements_currently_due: Vec<String>,
    pub requirements_eventually_due: Vec<String>,
}

/// Create Stripe Connect account and return onboarding URL
#[tracing::instrument(skip(state, req, user), fields(org_id = %req.org_id, user_id = %user.id))]
pub async fn create_onboarding(
    State(state): State<StripeState>,
    user: CurrentUser,
    Json(req): Json<CreateOnboardingRequest>,
) -> AppResult<(StatusCode, Json<CreateOnboardingResponse>)> {
    state.assert_org_member(req.org_id, user.id).await?;
    let (account_id, onboarding_url) = state.create_onboarding(req.org_id, req.entity_type, req.country).await?;
    
    Ok((StatusCode::CREATED, Json(CreateOnboardingResponse {
        account_id,
        onboarding_url,
    })))
}

/// Check Stripe onboarding status
pub async fn onboarding_status(
    State(state): State<StripeState>,
    user: CurrentUser,
    Path(org_id): Path<Uuid>,
) -> AppResult<Json<OnboardingStatusResponse>> {
    state.assert_org_member(org_id, user.id).await?;
    let status = state.check_onboarding_status(org_id).await?;
    
    Ok(Json(OnboardingStatusResponse {
        account_id: status.account_id,
        charges_enabled: status.charges_enabled,
        payouts_enabled: status.payouts_enabled,
        onboarding_complete: status.onboarding_complete,
        requirements_currently_due: status.requirements_currently_due,
        requirements_eventually_due: status.requirements_eventually_due,
    }))
}

/// Refresh onboarding link (generate new session)
pub async fn refresh_onboarding(
    State(state): State<StripeState>,
    user: CurrentUser,
    Path(org_id): Path<Uuid>,
) -> AppResult<Json<CreateOnboardingResponse>> {
    state.assert_org_member(org_id, user.id).await?;
    let org = state.org_repo.find_by_id(org_id).await?
        .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

    let account_id = org.stripe_account_id
        .ok_or_else(|| AppError::BadRequest("Stripe account not created yet".to_string()))?;

    let session_resp = state.client.create_account_session(&account_id, &state.frontend_base, org_id).await?;
    let onboarding_url = session_resp.url;

    Ok(Json(CreateOnboardingResponse {
        account_id,
        onboarding_url,
    }))
}

/// Stripe redirects here after onboarding completes
/// Query params: org_id (from our return_url)
#[derive(Deserialize)]
pub struct ReturnParams {
    pub org_id: Uuid,
}

/// Frontend success/error page URLs
fn frontend_success_url(frontend_base: &str, org_id: Uuid) -> String {
    format!("{}/onboarding/success?org_id={}", frontend_base, org_id)
}

fn frontend_error_url(frontend_base: &str, org_id: Uuid) -> String {
    format!("{}/onboarding/error?org_id={}", frontend_base, org_id)
}

/// Stripe onboarding return handler
/// Called when user completes onboarding and is redirected back
pub async fn onboarding_return(
    State(state): State<StripeState>,
    Query(params): Query<ReturnParams>,
) -> impl IntoResponse {
    let org_id = params.org_id;
    let frontend_base = state.frontend_base.clone();

    // Check onboarding status
    match state.check_onboarding_status(org_id).await {
        Ok(status) if status.onboarding_complete => {
            Redirect::to(&frontend_success_url(&frontend_base, org_id)).into_response()
        }
        Ok(_) => {
            // Not complete yet - redirect to error page, frontend can poll
            Redirect::to(&frontend_error_url(&frontend_base, org_id)).into_response()
        }
        Err(e) => {
            tracing::error!(org_id = %org_id, error = %e, "Onboarding status check failed");
            Redirect::to(&frontend_error_url(&frontend_base, org_id)).into_response()
        }
    }
}

/// Create a donation PaymentIntent
#[tracing::instrument(skip(state, req), fields(subdomain = %subdomain))]
pub async fn create_donation(
    State(state): State<StripeState>,
    Path(subdomain): Path<String>,
    Json(req): Json<CreateDonationRequest>,
) -> AppResult<(StatusCode, Json<crate::stripe::models::CreateDonationResponse>)> {
    let (client_secret, donation_id, payment_intent_id, platform_fee_cents) = 
        state.create_donation_payment_intent(&subdomain, req).await?;

    Ok((StatusCode::CREATED, Json(crate::stripe::models::CreateDonationResponse {
        client_secret,
        donation_id,
        payment_intent_id,
    })))
}

/// Get donation status
pub async fn donation_status(
    State(state): State<StripeState>,
    Path(donation_id): Path<Uuid>,
) -> AppResult<Json<crate::stripe::models::DonationStatusResponse>> {
    // TODO: Implement donation status lookup from database
    Err(AppError::Internal)
}

pub fn router(state: StripeState) -> Router {
    Router::new()
        .route("/onboarding/stripe/create", post(create_onboarding))
        .route("/onboarding/stripe/status/{org_id}", get(onboarding_status))
        .route("/onboarding/stripe/refresh/{org_id}", post(refresh_onboarding))
        .route("/onboarding/stripe/return", get(onboarding_return))
        .route("/orgs/{subdomain}/donate", post(create_donation))
        .route("/orgs/{subdomain}/donate/{donation_id}", get(donation_status))
        .with_state(state)
}
