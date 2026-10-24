use chrono::{Duration, Utc};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::auth::models::{NewUser, User, Tokens};
use crate::auth::ports::{AuthProvider, SessionRepository, UserRepository};
use crate::shared::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct AuthService<A: AuthProvider, U: UserRepository, S: SessionRepository> {
    auth_provider: A,
    user_repo: U,
    session_repo: S,
    refresh_token_expiry_days: i64,
}

impl<A: AuthProvider, U: UserRepository, S: SessionRepository> AuthService<A, U, S> {
    pub fn new(auth_provider: A, user_repo: U, session_repo: S, refresh_token_expiry_days: i64) -> Self {
        Self {
            auth_provider,
            user_repo,
            session_repo,
            refresh_token_expiry_days,
        }
    }

    fn new_refresh_token(&self) -> AppResult<(String, chrono::NaiveDateTime)> {
        let expires_at = Utc::now()
            .checked_add_signed(Duration::days(self.refresh_token_expiry_days))
            .ok_or(AppError::Internal)?
            .naive_utc();
        let token = Uuid::new_v4().to_string();
        Ok((token, expires_at))
    }

    async fn with_refresh(&self, user_id: Uuid, tokens: Tokens) -> AppResult<Tokens> {
        let (rt, expires_at) = self.new_refresh_token()?;
        self.session_repo.create(user_id, rt.clone(), expires_at).await?;
        Ok(Tokens {
            access_token: tokens.access_token,
            expires_in: tokens.expires_in,
            refresh_token: Some(rt),
        })
    }

    #[instrument(skip(self, password), fields(email = %email))]
    pub async fn register(
        &self,
        email: String,
        name: String,
        password: String,
    ) -> AppResult<(User, Tokens)> {
        let email = email.to_lowercase();
        if email.is_empty() {
            return Err(AppError::BadRequest("Email cannot be empty".to_string()));
        }
        if name.is_empty() {
            return Err(AppError::BadRequest("Name cannot be empty".to_string()));
        }
        if password.len() < 8 {
            return Err(AppError::BadRequest(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        if self.user_repo.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        let (user_info, tokens) = self
            .auth_provider
            .register(&email, &name, &password)
            .await?;

        let new_user = NewUser {
            id: Uuid::new_v4(),
            email,
            name,
            password_hash: user_info.password_hash,
        };

        let user = self.user_repo.create(new_user).await?;
        let tokens = self.with_refresh(user.id, tokens).await?;
        info!(user_id = %user.id, "user registered");
        Ok((user, tokens))
    }

    #[instrument(skip(self, password), fields(email = %email))]
    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> AppResult<(User, Tokens)> {
        let email = email.to_lowercase();
        let user = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or_else(|| {
                warn!(email = %email, "login failed: user not found");
                AppError::BadRequest("Invalid email or password".to_string())
            })?;

        let (tokens, _user_info) = self
            .auth_provider
            .login(&email, &password, &user.password_hash)
            .await?;

        let tokens = self.with_refresh(user.id, tokens).await?;
        info!(user_id = %user.id, "user logged in");
        Ok((user, tokens))
    }

    #[instrument(skip(self, refresh_token))]
    pub async fn logout(&self, refresh_token: String) -> AppResult<()> {
        self.session_repo.revoke(&refresh_token).await
    }

    #[instrument(skip(self, refresh_token))]
    pub async fn refresh_token(
        &self,
        refresh_token: String,
    ) -> AppResult<(User, Tokens)> {
        let session = self
            .session_repo
            .find_by_token(&refresh_token)
            .await?
            .ok_or_else(|| AppError::BadRequest("Invalid refresh token".to_string()))?;

        if session.expires_at < Utc::now().naive_utc() {
            self.session_repo.revoke(&refresh_token).await?;
            return Err(AppError::BadRequest("Refresh token expired".to_string()));
        }

        let user = self
            .user_repo
            .find_by_id(session.user_id)
            .await?
            .ok_or(AppError::Internal)?;

        let access_token = self
            .auth_provider
            .generate_access_token(&user.id.to_string(), &user.email)
            .await?;

        debug!("rotating refresh token");
        self.session_repo.revoke(&refresh_token).await?;
        let (new_rt, expires_at) = self.new_refresh_token()?;
        self.session_repo
            .create(user.id, new_rt.clone(), expires_at)
            .await?;

        let tokens = Tokens {
            access_token,
            expires_in: self.auth_provider.token_expiry_seconds(),
            refresh_token: Some(new_rt),
        };
        info!(user_id = %user.id, "token refreshed");
        Ok((user, tokens))
    }
}

pub type CreamAuthService = crate::auth::service::AuthService<
    crate::auth::provider::LocalAuthProvider,
    crate::auth::repository::DieselUserRepository,
    crate::auth::repository::DieselSessionRepository,
>;
