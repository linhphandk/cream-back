use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::models::UserInfo;
use crate::auth::ports::AuthProvider;
use crate::shared::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct LocalAuthProvider {
    pub jwt_secret: String,
    pub access_token_expiry_minutes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub organization_id: Option<String>,
    pub exp: usize,
}

impl LocalAuthProvider {
    pub fn new(jwt_secret: String, access_token_expiry_minutes: u64) -> Self {
        Self {
            jwt_secret,
            access_token_expiry_minutes,
        }
    }

    pub fn generate_access_token(
        &self,
        user_id: &str,
        email: &str,
        organization_id: Option<Uuid>,
    ) -> AppResult<String> {
        let exp = Utc::now()
            .checked_add_signed(Duration::minutes(self.access_token_expiry_minutes as i64))
            .ok_or(AppError::Internal)?
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            organization_id: organization_id.map(|id| id.to_string()),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| {
            tracing::error!("JWT encoding error: {:?}", e);
            AppError::Internal
        })
    }
}

#[async_trait]
impl AuthProvider for LocalAuthProvider {
    async fn hash_password(&self, password: &str) -> AppResult<String> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| {
            tracing::error!("Password hashing error: {:?}", e);
            AppError::Internal
        })
    }

    async fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        bcrypt::verify(password, hash).map_err(|e| {
            tracing::error!("Password verification error: {:?}", e);
            AppError::Internal
        })
    }

    async fn introspect_token(&self, token: &str) -> AppResult<UserInfo> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::error!("JWT decode error: {:?}", e);
            AppError::Unauthorized("Invalid token".to_string())
        })?;

        Ok(UserInfo {
            sub: token_data.claims.sub,
            email: token_data.claims.email,
            name: String::new(),
            organization_id: token_data
                .claims
                .organization_id
                .and_then(|s| s.parse::<Uuid>().ok()),
            password_hash: String::new(),
        })
    }

    fn token_expiry_seconds(&self) -> u64 {
        self.access_token_expiry_minutes * 60
    }
}