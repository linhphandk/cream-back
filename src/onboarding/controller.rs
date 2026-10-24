use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::organizations::models::{CreateOrganizationRequest, Organization};
use crate::shared::errors::AppError;

use super::service::OnboardingService;

pub type OnboardingAppState = std::sync::Arc<OnboardingService>;

#[tracing::instrument(skip(state, body), fields(subdomain = %body.subdomain))]
pub async fn create_organization(
    State(state): State<OnboardingAppState>,
    Json(body): Json<CreateOrganizationRequest>,
) -> Result<(StatusCode, Json<Organization>), AppError> {
    let org = state.create_organization(body).await?;
    Ok((StatusCode::CREATED, Json(org)))
}
