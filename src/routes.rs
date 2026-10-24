use std::sync::Arc;

use axum::Router;

use crate::auth;
use crate::auth::service::CreamAuthService;
use crate::config::Settings;
use crate::organizations;
use crate::shared::db::DbPool;

pub fn auth_state(pool: DbPool, settings: &Settings) -> Arc<CreamAuthService> {
    let auth_provider = auth::provider::LocalAuthProvider::new(
        settings.jwt_secret.clone(),
        settings.jwt_expiry_minutes as u64,
    );
    let user_repo = auth::repository::DieselUserRepository::new(pool.clone());
    let session_repo = auth::repository::DieselSessionRepository::new(pool);
    Arc::new(auth::service::AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        7,
    ))
}

pub fn routes(
    auth_service: Arc<CreamAuthService>,
) -> Router {
    Router::new()
        .route("/health", axum::routing::get(|| async { "healthy" }))
        .merge(auth::router(auth_service))
}
