use diesel::prelude::*;
use tracing::instrument;

use crate::organizations::models::{Campaign, NewCampaign, NewOrganization, Organization};
use crate::shared::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct OrgRepository {
    pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>,
}

impl OrgRepository {
    pub fn new(pool: crate::shared::db::DbPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self, org), fields(subdomain = %org.subdomain))]
    pub async fn create(
        &self,
        org: NewOrganization,
        default_campaign: NewCampaign,
    ) -> AppResult<(Organization, Campaign)> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("pool error: {:?}", e);
            AppError::Internal
        })?;

        conn.transaction::<(Organization, Campaign), diesel::result::Error, _>(|conn| {
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
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => AppError::Conflict("subdomain already taken".to_string()),
                _ => AppError::Internal,
            }
        })
    }
}
