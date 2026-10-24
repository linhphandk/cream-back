pub mod controller;
pub mod service;

use axum::routing::post;

pub fn router(state: controller::OnboardingAppState) -> axum::Router {
    axum::Router::new()
        .route("/orgs", post(controller::create_organization))
        .with_state(state)
}
