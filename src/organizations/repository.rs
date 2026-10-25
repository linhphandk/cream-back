use uuid::Uuid;
use diesel::prelude::*;
use tracing::instrument;

use crate::organizations::models::{
    Campaign, NewCampaign, NewOrganization, NewOrganizationMember, Organization,
    OrganizationMember,
};
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
        owner_id: Uuid,
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

            let member = NewOrganizationMember {
                org_id: org.id,
                user_id: owner_id,
                role: "owner".to_string(),
            };
            diesel::insert_into(crate::schema::organization_members::table)
                .values(&member)
                .execute(conn)?;

            Ok((org, campaign))
        })
        .map_err(|e| {
            tracing::error!("org create: {:?}", e);
            match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    info,
                ) => {
                    if info.message().contains("organization_members_pkey") {
                        AppError::Conflict("user already owns an organization".to_string())
                    } else if info.message().contains("organization_members_one_owner") {
                        AppError::Conflict("organization already has an owner".to_string())
                    } else {
                        AppError::Conflict("subdomain already taken".to_string())
                    }
                }
                _ => AppError::Internal,
            }
        })
    }

    /// The org a user belongs to (one per user, enforced by the members PK).
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn find_by_member(&self, user_id: Uuid) -> AppResult<Option<Organization>> {
        use crate::schema::organization_members;
        use crate::schema::organizations;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        organizations::table
            .inner_join(organization_members::table)
            .filter(organization_members::user_id.eq(user_id))
            .select(Organization::as_select())
            .first::<Organization>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("org by member: {:?}", e);
                AppError::Internal
            })
    }

    pub async fn find_member(
        &self,
        org: Uuid,
        user: Uuid,
    ) -> AppResult<Option<OrganizationMember>> {
        use crate::schema::organization_members::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        organization_members
            .filter(org_id.eq(org).and(user_id.eq(user)))
            .select(OrganizationMember::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("member lookup: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self), fields(subdomain = %sub))]
    pub async fn find_by_subdomain(&self, sub: &str) -> AppResult<Option<Organization>> {
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

    pub async fn find_by_id(&self, org_id: Uuid) -> AppResult<Option<Organization>> {
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

    pub async fn update_stripe_account_id(&self, org_id: Uuid, account_id: String) -> AppResult<()> {
        use crate::schema::organizations::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        diesel::update(organizations.filter(id.eq(org_id)))
            .set(stripe_account_id.eq(account_id))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("update stripe account id: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }

    pub async fn update_stripe_status(
        &self,
        org_id: Uuid,
        charges_enabled: bool,
        payouts_enabled: bool,
        onboarding_complete: bool,
        status: &str,
    ) -> AppResult<()> {
        use crate::schema::organizations::dsl::*;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        diesel::update(organizations.filter(id.eq(org_id)))
            .set((
                stripe_charges_enabled.eq(charges_enabled),
                stripe_payouts_enabled.eq(payouts_enabled),
                stripe_onboarding_complete.eq(onboarding_complete),
                stripe_account_status.eq(status),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("update stripe status: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }
}
