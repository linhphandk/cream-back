use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::auth::models::{Tokens, UserInfo};
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
    pub exp: usize,
}

impl LocalAuthProvider {
    pub fn new(jwt_secret: String, access_token_expiry_minutes: u64) -> Self {
        Self {
            jwt_secret,
            access_token_expiry_minutes,
        }
    }

    fn generate_access_token(&self, user_id: &str, email: &str) -> AppResult<String> {
        let exp = Utc::now()
            .checked_add_signed(Duration::minutes(self.access_token_expiry_minutes as i64))
            .ok_or(AppError::Internal)?
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
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

    async fn issue_tokens(&self, user: &UserInfo) -> AppResult<Tokens> {
        let access_token = self.generate_access_token(&user.sub, &user.email)?;

        Ok(Tokens {
            access_token,
            expires_in: self.access_token_expiry_minutes * 60,
            refresh_token: None,
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
            password_hash: String::new(),
        })
    }

    fn token_expiry_seconds(&self) -> u64 {
        self.access_token_expiry_minutes * 60
    }

    fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}
