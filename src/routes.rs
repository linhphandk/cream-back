use axum::Router;
use std::sync::Arc;

use crate::auth::controller;
use crate::auth::ports::OrganizationRepository;
use crate::auth::service::OrganizationService;

pub fn routes<R: OrganizationRepository + 'static>(
    org_service: Arc<OrganizationService<R>>,
) -> Router {
    Router::new()
        .route(
            "/api/orgs",
            axum::routing::post(controller::create_organization::<R>),
        )
        .route(
            "/api/orgs/{subdomain}",
            axum::routing::get(controller::get_organization::<R>),
        )
        .route("/health", axum::routing::get(|| async { "healthy" }))
        .with_state((*org_service).clone())
}
