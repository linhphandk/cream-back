use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;

use crate::auth::models::{NewOrganization, Organization};
use crate::auth::ports::OrganizationRepository;
use crate::shared::errors::{AppError, AppResult};

type PgPool = Pool<ConnectionManager<PgConnection>>;

pub struct PostgresOrganizationRepository {
    pool: PgPool,
}

impl PostgresOrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRepository for PostgresOrganizationRepository {
    async fn create(&self, org: NewOrganization) -> AppResult<Organization> {
        use crate::schema::organizations::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        diesel::insert_into(organizations)
            .values(&org)
            .get_result::<Organization>(&mut conn)
            .map_err(|e| {
                tracing::error!("DB error creating organization: {:?}", e);
                AppError::Internal
            })
    }

    async fn find_by_subdomain(&self, sub: &str) -> AppResult<Option<Organization>> {
        use crate::schema::organizations::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        organizations
            .filter(subdomain.eq(sub))
            .first::<Organization>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("DB error finding org by subdomain: {:?}", e);
                AppError::Internal
            })
    }

    async fn find_by_id(&self, org_id: Uuid) -> AppResult<Option<Organization>> {
        use crate::schema::organizations::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        organizations
            .filter(id.eq(org_id))
            .first::<Organization>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("DB error finding org by id: {:?}", e);
                AppError::Internal
            })
    }

    async fn update_stripe_account_id(
        &self,
        org_id: Uuid,
        account_id: String,
    ) -> AppResult<()> {
        use crate::schema::organizations::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        diesel::update(organizations.filter(id.eq(org_id)))
            .set(stripe_account_id.eq(account_id))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB error updating stripe_account_id: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }
}