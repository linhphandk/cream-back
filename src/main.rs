mod auth;
mod schema;
mod shared;
mod config;
mod routes;

use std::sync::Arc;

use config::Settings;
use tracing::info;

use crate::shared::db::{establish_pool, DbPool};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cream_back=debug,tower_http=debug".parse().unwrap()),
        )
        .init();

    info!("starting cream-back");

    let config_state: Settings = envy::from_env().unwrap();

    let pool = establish_pool(&config_state.database_url, config_state.db_pool_size);

    let auth_service = build_auth_service(pool, &config_state);

    let app = routes::routes(auth_service);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    info!(addr, "serving");

    axum::serve(listener, app).await.unwrap();
}

fn build_auth_service(
    pool: DbPool,
    config_state: &Settings,
) -> Arc<crate::auth::service::CreamAuthService> {
    let auth_provider = crate::auth::provider::LocalAuthProvider::new(
        config_state.jwt_secret.clone(),
        config_state.jwt_expiry_minutes as u64,
    );
    let user_repo = crate::auth::repository::DieselUserRepository::new(pool.clone());
    let session_repo = crate::auth::repository::DieselSessionRepository::new(pool);
    Arc::new(crate::auth::service::AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        7,
    ))
}
