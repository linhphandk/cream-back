pub mod controller;
pub mod extractor;
pub mod models;
pub mod ports;
pub mod provider;
pub mod repository;
pub mod service;

use axum::routing::post;
use std::sync::Arc;

pub type CreamAppState = Arc<crate::auth::service::CreamAuthService>;

pub fn router(state: CreamAppState) -> axum::Router {
    axum::Router::new()
        .route("/auth/register", post(controller::register))
        .route("/auth/login", post(controller::login))
        .route("/auth/logout", post(controller::logout))
        .route("/auth/refresh", post(controller::refresh))
        .with_state(state)
}
