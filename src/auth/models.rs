use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Tokens {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
fn org_later(_org_id: Option<Uuid>) {}
