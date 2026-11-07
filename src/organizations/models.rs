use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone, Debug, Serialize)]
#[diesel(table_name = crate::schema::organizations)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub subdomain: String,
    pub fee_bps: i32,
    pub stripe_account_id: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub stripe_charges_enabled: Option<bool>,
    pub stripe_payouts_enabled: Option<bool>,
    pub stripe_details_submitted: Option<bool>,
}

impl Organization {
    /// Cached Stripe view says the account can fully operate.
    pub fn stripe_complete(&self) -> bool {
        self.stripe_charges_enabled.unwrap_or(false)
            && self.stripe_payouts_enabled.unwrap_or(false)
            && self.stripe_details_submitted.unwrap_or(false)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::organizations)]
pub struct NewOrganization {
    pub id: Uuid,
    pub name: String,
    pub subdomain: String,
    pub fee_bps: i32,
    pub stripe_account_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub subdomain: String,
    #[serde(default = "default_fee_bps")]
    pub fee_bps: i32,
}

fn default_fee_bps() -> i32 {
    500
}

#[derive(Queryable, Selectable, Clone, Debug, Serialize)]
#[diesel(table_name = crate::schema::campaigns)]
#[serde(rename_all = "camelCase")]
pub struct Campaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub currency: String,
    pub active: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::campaigns)]
pub struct NewCampaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub currency: String,
    pub active: Option<bool>,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::organization_members)]
pub struct OrganizationMember {
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::organization_members)]
pub struct NewOrganizationMember {
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
}

fn default_currency() -> String {
    "USD".to_string()
}
