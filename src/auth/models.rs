use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Tokens {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub organization_id: Option<Uuid>,
    pub password_hash: String,
}

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
pub struct RegisterOrganizationRequest {
    pub name: String,
    pub subdomain: String,
    #[serde(default = "default_fee_bps")]
    pub fee_bps: i32,
    pub email: String,
    pub password: String,
}

fn default_fee_bps() -> i32 {
    500
}