use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Tokens {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::sessions)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
