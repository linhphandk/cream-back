use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry_minutes: u32,
    #[serde(default = "default_pool")]
    pub db_pool_size: u32,
}

fn default_jwt_expiry() -> u32 { 15 }
fn default_pool() -> u32 { 10 }
