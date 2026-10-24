pub mod client;
pub mod controller;
pub mod models;
pub mod service;
pub mod webhooks;

use crate::config::Settings;
use crate::shared::errors::AppResult;

pub use client::StripeClient;
pub use service::StripeService;

/// Initialize Stripe client from settings
pub fn init_client(settings: &Settings) -> AppResult<StripeClient> {
    StripeClient::new(
        settings.stripe_secret_key.clone(),
        settings.stripe_webhook_secret.clone(),
        settings.stripe_api_version.clone(),
    )
}
