pub mod controller;
pub mod models;
pub mod repository;
pub mod service;

use axum::routing::post;
use std::sync::Arc;

use crate::organizations::controller as c;

pub use controller::OrgState;

pub fn router(state: OrgState) -> axum::Router {
    axum::Router::new()
        .route("/orgs", post(c::create))
        .with_state(state)
}
