use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::Serialize;
use tracing::instrument;

use crate::auth::models::{LoginRequest, RegisterRequest};
use crate::shared::errors::{AppError, AppResult};

use super::service::CreamAuthService;

const REFRESH_COOKIE_NAME: &str = "refresh_token";
const REFRESH_COOKIE_MAX_AGE: &str = "604800"; // 7 days

fn refresh_cookie(value: &str) -> String {
    format!(
        "{name}={value}; HttpOnly; SameSite=Strict; Path=/auth; Max-Age={max_age}",
        name = REFRESH_COOKIE_NAME,
        value = value,
        max_age = REFRESH_COOKIE_MAX_AGE,
    )
}

fn clear_refresh_cookie() -> String {
    format!(
        "{name}=; HttpOnly; SameSite=Strict; Path=/auth; Max-Age=0",
        name = REFRESH_COOKIE_NAME,
    )
}

fn extract_refresh_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let mut kv = part.trim().splitn(2, '=');
        let key = kv.next()?.trim();
        let val = kv.next()?.trim();
        if key == REFRESH_COOKIE_NAME {
            return Some(val.to_string());
        }
    }
    None
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
}

type AuthState = std::sync::Arc<CreamAuthService>;

#[instrument(skip(state, req), fields(email = %req.email))]
pub async fn register(
    axum::extract::State(state): axum::extract::State<AuthState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> AppResult<(StatusCode, HeaderMap, Json<AuthResponse>)> {
    let (user, tokens) = state
        .clone()
        .register(req.email, req.name, req.password)
        .await?;
    let response = AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        },
        access_token: tokens.access_token,
        expires_in: tokens.expires_in,
    };
    let mut resp_headers = HeaderMap::new();
    if let Some(rt) = &tokens.refresh_token {
        resp_headers.insert(SET_COOKIE, refresh_cookie(rt).parse().unwrap());
    }
    Ok((StatusCode::CREATED, resp_headers, Json(response)))
}

#[instrument(skip(state, req), fields(email = %req.email))]
pub async fn login(
    axum::extract::State(state): axum::extract::State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<(StatusCode, HeaderMap, Json<AuthResponse>)> {
    let (user, tokens) = state.clone().login(req.email, req.password).await?;
    let response = AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        },
        access_token: tokens.access_token,
        expires_in: tokens.expires_in,
    };
    let mut resp_headers = HeaderMap::new();
    if let Some(rt) = &tokens.refresh_token {
        resp_headers.insert(SET_COOKIE, refresh_cookie(rt).parse().unwrap());
    }
    Ok((StatusCode::OK, resp_headers, Json(response)))
}

#[instrument(skip(state, headers))]
pub async fn logout(
    axum::extract::State(state): axum::extract::State<AuthState>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, HeaderMap)> {
    if let Some(rt) = extract_refresh_token(&headers) {
        let _ = state.clone().logout(rt).await;
    }
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(SET_COOKIE, clear_refresh_cookie().parse().unwrap());
    Ok((StatusCode::OK, resp_headers))
}

#[instrument(skip(state, headers))]
pub async fn refresh(
    axum::extract::State(state): axum::extract::State<AuthState>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, HeaderMap, Json<AuthResponse>)> {
    let rt = extract_refresh_token(&headers)
        .ok_or_else(|| AppError::Unauthorized("Missing refresh token cookie".to_string()))?;
    let (user, tokens) = state.clone().refresh_token(rt).await?;
    let response = AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        },
        access_token: tokens.access_token,
        expires_in: tokens.expires_in,
    };
    let mut resp_headers = HeaderMap::new();
    if let Some(rt) = &tokens.refresh_token {
        resp_headers.insert(SET_COOKIE, refresh_cookie(rt).parse().unwrap());
    }
    Ok((StatusCode::OK, resp_headers, Json(response)))
}
