mod auth;
mod config;
mod onboarding;
mod organizations;
mod routes;
mod schema;
mod shared;
mod stripe;

use std::sync::Arc;

use tracing::info;

use crate::config::Settings;
use crate::stripe::StripeClient;

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

    // Stripe client
    let stripe_client = Arc::new(StripeClient::new(
        settings.stripe_secret_key.clone(),
        settings.stripe_webhook_secret.clone(),
        settings.stripe_api_version.clone(),
    ).expect("Failed to create Stripe client"));

    // Auth
    let auth_service = routes::auth_state(pool.clone(), &settings);
    
    // Organizations
    let org_repo = Arc::new(organizations::repository::OrgRepository::new(pool.clone()));
    let org_service = Arc::new(organizations::service::OrganizationService::new((*org_repo).clone()));
    
    // Onboarding
    let onboarding: onboarding::controller::OnboardingAppState = Arc::new(
        onboarding::service::OnboardingService::new(org_service.clone()),
    );
    
    // Stripe service
    let stripe_service = Arc::new(stripe::service::StripeService::new(
        stripe_client,
        org_repo,
        settings.frontend_base_url.clone(),
        settings.platform_fee_bps,
    ));

    let app = routes::routes(auth_service, onboarding, stripe_service);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    info!(addr, "serving");

    axum::serve(listener, app).await.unwrap();
}
