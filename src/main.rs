mod auth;
mod config;
mod organizations;
mod routes;
mod schema;
mod shared;

use std::sync::Arc;

use tracing::info;

use crate::config::Settings;

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

    let settings: Settings = envy::from_env().unwrap();
    let pool = shared::db::establish_pool(&settings.database_url, settings.db_pool_size);

    let auth_service = routes::auth_state(pool.clone(), &settings);
    let org_service: organizations::OrgState = Arc::new(organizations::service::OrganizationService::new(
        organizations::repository::OrgRepository::new(pool),
    ));

    let app = routes::routes(auth_service, org_service);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    info!(addr, "serving");

    axum::serve(listener, app).await.unwrap();
}
