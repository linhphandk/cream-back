use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

use crate::organizations::models::{CreateOrganizationRequest, Organization};
use crate::organizations::service::OrganizationService;
use crate::shared::errors::AppError;

pub type OrgState = Arc<OrganizationService>;

#[tracing::instrument(skip(state, body), fields(subdomain = %body.subdomain))]
pub async fn create(
    State(state): State<OrgState>,
    Json(body): Json<CreateOrganizationRequest>,
) -> Result<(StatusCode, Json<Organization>), AppError> {
    let org = state.create(body).await?;
    Ok((StatusCode::CREATED, Json(org)))
}
