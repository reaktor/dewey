use chrono::{DateTime, Utc};
use crate::user;

use super::schema::users;
use super::schema::user_tokens;

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub google_resource_id: Option<&'a str>,
    pub full_name: &'a str,
    pub display_name: &'a str,
    pub public_email: Option<&'a str>,
    pub kind: &'a user::UserKind,
    pub photo_url: Option<&'a str>,
}

#[derive(Queryable)]
pub struct UserToken {
    pub user_id: user::UserId,
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
    pub user_id: user::UserId,
    pub google_resource_id: &'a str,
    pub version: i32,
    pub access_token: &'a str,
    pub refresh_token: &'a str,
    pub token_expiration: &'a DateTime<Utc>,
}
