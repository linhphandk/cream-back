use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Json as AxumJson;
use serde_json::json;
use uuid::Uuid;

use crate::auth::models::{Organization, RegisterOrganizationRequest};
use crate::auth::service::OrganizationService;
use crate::shared::errors::AppError;

fn error_response(e: AppError) -> axum::response::Response {
    let (status, message) = match &e {
        AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
        AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
        AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
        AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
        AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
        AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
    };
    (status, AxumJson(json!({ "error": message }))).into_response()
}

pub async fn create_organization<S>(
    State(service): State<OrganizationService<S>>,
    AxumJson(body): AxumJson<RegisterOrganizationRequest>,
) -> Result<axum::response::Response, axum::response::Response>
where
    S: crate::auth::ports::OrganizationRepository,
{
    service
        .create_organization(&body.name, &body.subdomain, body.fee_bps)
        .await
        .map(|org: Organization| (StatusCode::CREATED, AxumJson(org)).into_response())
        .map_err(|e| error_response(e))
}

pub async fn get_organization<S>(
    State(service): State<OrganizationService<S>>,
    axum::extract::Path(subdomain): axum::extract::Path<String>,
) -> Result<axum::response::Response, axum::response::Response>
where
    S: crate::auth::ports::OrganizationRepository,
{
    service
        .get_organization_by_subdomain(&subdomain)
        .await
        .map(|org| AxumJson(org).into_response())
        .map_err(|e| error_response(e))
}

// keep Uuid import used for org ids in future handlers
#[allow(dead_code)]
fn _uuid_used(_u: Uuid) {}
