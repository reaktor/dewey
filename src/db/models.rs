use chrono::{DateTime, TimeZone, Utc};

use super::schema::users;
use super::schema::user_tokens;
use super::schema::UserKind;

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i64,
    pub google_resource_id: Option<String>,
    pub full_name: String,
    pub display_name: String,
    pub public_email: Option<String>,
    pub kind: UserKind,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub google_resource_id: Option<&'a str>,
    pub full_name: &'a str,
    pub display_name: &'a str,
    pub public_email: Option<&'a str>,
    pub kind: &'a UserKind,
}

#[derive(Queryable)]
pub struct UserToken {
    pub user_id: i64,
    pub google_resource_id: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiration: DateTime<Utc>,
}

#[derive(Insertable)]
#[table_name="user_tokens"]
pub struct NewUserToken<'a> {
    pub user_id: i64,
    pub google_resource_id: &'a str,
    pub version: i32,
    pub access_token: &'a str,
    pub refresh_token: &'a str,
    pub token_expiration: &'a DateTime<Utc>,
}

#[derive(Queryable)]
pub struct Object {
    pub id: String,
    pub extension: String,
    pub created_by: i64,
    pub created_at: std::time::SystemTime,
}
