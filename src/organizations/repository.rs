use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use tracing::instrument;
use uuid::Uuid;

use crate::organizations::models::{Campaign, NewCampaign, NewOrganization, Organization};
use crate::shared::errors::{AppError, AppResult};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct OrgRepository {
    pool: DbPool,
}

impl OrgRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl super::ports::OrganizationRepository for OrgRepository {
    #[instrument(skip(self, org), fields(subdomain = %org.subdomain))]
    async fn create(&self, org: NewOrganization, default_campaign: NewCampaign) -> AppResult<(Organization, Campaign)> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("pool error: {:?}", e);
            AppError::Internal
        })?;

        let (org, campaign) = conn
            .transaction::<(Organization, Campaign), diesel::result::Error, _>(|conn| {
                let org: Organization = diesel::insert_into(crate::schema::organizations::table)
                    .values(&org)
                    .get_result(conn)?;

                // silent default campaign: single form per org, multicampaign later
                let campaign: Campaign = diesel::insert_into(crate::schema::campaigns::table)
                    .values(&default_campaign)
                    .get_result(conn)?;

                Ok((org, campaign))
            })
            .map_err(|e| {
                tracing::error!("org create: {:?}", e);
                match e {
                    diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => {
                        AppError::Conflict("subdomain already taken".to_string())
                    }
                    _ => AppError::Internal,
                }
            })?;

        Ok((org, campaign))
    }

    #[instrument(skip(self), fields(subdomain = %sub))]
    async fn find_by_subdomain(&self, sub: &str) -> AppResult<Option<Organization>> {
        use crate::schema::organizations::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        organizations
            .filter(subdomain.eq(sub))
            .first::<Organization>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("org by subdomain: {:?}", e);
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
                tracing::error!("org by id: {:?}", e);
                AppError::Internal
            })
    }

    async fn find_campaign_for_org(&self, org_id: Uuid) -> AppResult<Option<Campaign>> {
        use crate::schema::campaigns::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        campaigns
            .filter(organization_id.eq(org_id))
            .first::<Campaign>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("campaign for org: {:?}", e);
                AppError::Internal
            })
    }

    async fn update_stripe_account_id(&self, org_id: Uuid, account_id: String) -> AppResult<()> {
        use crate::schema::organizations::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        diesel::update(organizations.filter(id.eq(org_id)))
            .set(stripe_account_id.eq(account_id))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("update stripe id: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }
}
