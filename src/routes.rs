use std::sync::Arc;

use axum::Router;

use crate::auth;

pub fn routes(auth_service: Arc<auth::service::CreamAuthService>) -> axum::Router {
    Router::new().merge(auth::router(auth_service))
}
