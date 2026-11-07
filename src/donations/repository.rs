use diesel::prelude::*;
use tracing::instrument;
use uuid::Uuid;

use crate::donations::models::{Donation, NewDonation};
use crate::shared::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct DonationRepository {
    pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>,
}

impl DonationRepository {
    pub fn new(pool: crate::shared::db::DbPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self, donation))]
    pub async fn create(&self, donation: NewDonation) -> AppResult<Donation> {
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        diesel::insert_into(crate::schema::donations::table)
            .values(&donation)
            .get_result(&mut conn)
            .map_err(|e| {
                tracing::error!("donation insert: {:?}", e);
                AppError::Internal
            })
    }

    /// Donation lookup scoped to the org's public subdomain — donors must not
    /// be able to probe other orgs' donation ids.
    #[instrument(skip(self), fields(donation_id = %donation_id, sub = %sub))]
    pub async fn find_by_id_and_subdomain(
        &self,
        donation_id: Uuid,
        sub: &str,
    ) -> AppResult<Option<Donation>> {
        use crate::schema::campaigns;
        use crate::schema::donations;
        use crate::schema::organizations;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        donations::table
            .inner_join(campaigns::table.inner_join(organizations::table))
            .filter(donations::id.eq(donation_id))
            .filter(organizations::subdomain.eq(sub))
            .select(Donation::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("donation lookup: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self), fields(donation_id = %donation_id, new_status = %new_status))]
    pub async fn update_status(
        &self,
        donation_id: Uuid,
        new_status: &str,
        payment_intent_id: Option<&str>,
    ) -> AppResult<()> {
        use crate::schema::donations;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;
        let target = donations::table.filter(donations::id.eq(donation_id));

        let res = match payment_intent_id {
            Some(pi) => diesel::update(target)
                .set((
                    donations::status.eq(new_status.to_string()),
                    donations::stripe_payment_intent_id.eq(pi.to_string()),
                ))
                .execute(&mut conn),
            None => diesel::update(target)
                .set(donations::status.eq(new_status.to_string()))
                .execute(&mut conn),
        };

        res.map_err(|e| {
            tracing::error!("donation status update: {:?}", e);
            AppError::Internal
        })?;
        Ok(())
    }

    /// Record a processed Stripe event. Returns false when the event id was
    /// already stored (unique index) so callers can skip duplicate deliveries.
    #[instrument(skip(self, payload))]
    pub async fn insert_event(
        &self,
        stripe_event_id: &str,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> AppResult<bool> {
        use crate::schema::stripe_webhook_events;
        let mut conn = self.pool.get().map_err(|_| AppError::Internal)?;

        let new_event = (
            stripe_webhook_events::stripe_event_id.eq(stripe_event_id),
            stripe_webhook_events::event_type.eq(event_type),
            stripe_webhook_events::payload.eq(payload),
        );

        let inserted = diesel::insert_into(stripe_webhook_events::table)
            .values(&new_event)
            .execute(&mut conn);

        match inserted {
            Ok(_) => Ok(true),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => Ok(false),
            Err(e) => {
                tracing::error!("webhook event insert: {:?}", e);
                Err(AppError::Internal)
            }
        }
    }
}
