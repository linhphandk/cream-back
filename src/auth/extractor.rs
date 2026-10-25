use std::sync::OnceLock;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use super::provider;
use crate::shared::errors::AppError;

static JWT_SECRET: OnceLock<String> = OnceLock::new();

/// Called once at startup; the secret is a process-wide constant anyway.
pub fn init(jwt_secret: String) {
    let _ = JWT_SECRET.set(jwt_secret);
}

/// Resolved from `Authorization: Bearer <jwt>` by handlers that declare it.
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
}

impl<S: Send + Sync> FromRequestParts<S> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("missing bearer token".to_string()))?;

        let secret = JWT_SECRET.get().ok_or(AppError::Internal)?;
        let (sub, email) = provider::verify_access_token(secret, token)?;
        let id = Uuid::parse_str(&sub)
            .map_err(|_| AppError::Unauthorized("invalid token subject".to_string()))?;
        Ok(CurrentUser { id, email })
    }
}
