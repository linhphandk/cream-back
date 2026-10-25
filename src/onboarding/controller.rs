use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::auth::extractor::CurrentUser;
use crate::organizations::models::{CreateOrganizationRequest, Organization};
use crate::shared::errors::AppError;

use super::service::OnboardingService;

pub type OnboardingAppState = std::sync::Arc<OnboardingService>;

#[tracing::instrument(skip(state, body, user), fields(subdomain = %body.subdomain, user_id = %user.id))]
pub async fn create_organization(
    State(state): State<OnboardingAppState>,
    user: CurrentUser,
    Json(body): Json<CreateOrganizationRequest>,
) -> Result<(StatusCode, Json<Organization>), AppError> {
    let org = state.create_organization(body, user.id).await?;
    Ok((StatusCode::CREATED, Json(org)))
}

/// The caller's org — or 404 if none yet.
pub async fn my_org(
    State(state): State<OnboardingAppState>,
    user: CurrentUser,
) -> Result<Json<Organization>, AppError> {
    let org = state
        .my_org(user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("no organization yet".to_string()))?;
    Ok(Json(org))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicOrgResponse {
    pub id: Uuid,
    pub name: String,
    pub subdomain: String,
    pub stripe_onboarding_complete: Option<bool>,
}

/// Public org info for the donate page.
#[tracing::instrument(skip(state), fields(subdomain = %sub))]
pub async fn public_org(
    State(state): State<OnboardingAppState>,
    Path(sub): Path<String>,
) -> Result<Json<PublicOrgResponse>, AppError> {
    let org = state
        .org_by_subdomain(&sub)
        .await?
        .ok_or_else(|| AppError::NotFound("organization not found".to_string()))?;
    Ok(Json(PublicOrgResponse {
        id: org.id,
        name: org.name,
        subdomain: org.subdomain,
        stripe_onboarding_complete: org.stripe_onboarding_complete,
    }))
}

pub fn router(state: OnboardingAppState) -> axum::Router {
    axum::Router::new()
        .route("/orgs", axum::routing::post(create_organization))
        .route("/orgs/mine", get(my_org))
        .route("/orgs/{subdomain}", get(public_org))
        .with_state(state)
}
