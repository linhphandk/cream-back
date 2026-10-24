use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry_minutes: u32,
    #[serde(default = "default_pool")]
    pub db_pool_size: u32,
    
    // Stripe
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    #[serde(default = "default_stripe_api_version")]
    pub stripe_api_version: String,
    #[serde(default = "default_frontend_base")]
    pub frontend_base_url: String,
    #[serde(default = "default_platform_fee_bps")]
    pub platform_fee_bps: i32,
}

fn default_jwt_expiry() -> u32 { 15 }
fn default_pool() -> u32 { 10 }
fn default_stripe_api_version() -> String { "2026-09-30.preview".to_string() }
fn default_frontend_base() -> String { "http://localhost:3000".to_string() }
fn default_platform_fee_bps() -> i32 { 500 }
