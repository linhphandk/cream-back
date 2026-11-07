use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone, Debug, Serialize)]
#[diesel(table_name = crate::schema::donations)]
#[serde(rename_all = "camelCase")]
pub struct Donation {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub stripe_payment_intent_id: Option<String>,
    pub donor_email: Option<String>,
    pub donor_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<NaiveDateTime>,
    pub stripe_transfer_id: Option<String>,
    pub platform_fee_cents: Option<i64>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::donations)]
pub struct NewDonation {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub platform_fee_cents: i64,
    pub donor_email: Option<String>,
    pub donor_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
