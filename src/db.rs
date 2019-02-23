//! Db executor actor
use ::actix::prelude::*;
use actix_web::*;
use actix_web::{error, Error, Result};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::Connection;
use diesel::PgConnection;

use crate::google_oauth::GoogleAccessToken;

mod models;
mod schema;
mod views;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

fn db_error<T: Into<String>>(message: T) -> Error {
    std::io::Error::new(std::io::ErrorKind::Other, message.into()).into()
}

/// Upsert user information, returning the new or existing user's id and version
pub struct UpsertGoogleUser {
    pub resource_id: String,
    pub full_name: String,
    pub display_name: String,
    pub access_token: GoogleAccessToken,
    pub refresh_token: String,
}

/// The created user's information
pub struct UserIdAndTokenVersion(pub i64, pub i32);

impl Message for UpsertGoogleUser {
    type Result = Result<UserIdAndTokenVersion>;
}

impl Handler<UpsertGoogleUser> for DbExecutor {
    type Result = Result<UserIdAndTokenVersion>;

    fn handle(&mut self, msg: UpsertGoogleUser, _: &mut Self::Context) -> Self::Result {
        use views::{get_user_token_versions_by_resource, ViewUserIdTokenVersion};
        let conn = self.0.get().unwrap();

        // Delete previous user tokens if they exist
        let user_exists: Option<ViewUserIdTokenVersion> =
            get_user_token_versions_by_resource(&conn, &msg.resource_id)
                .map_err(|_| db_error("db select view of user token version error"))?;

        let (user_id, version): (i64, i32) = match user_exists {
            Some(token_version) => {
                // User exists
                use schema::user_tokens::dsl::*;
                // Delete previous user tokens
                diesel::delete(user_tokens.filter(user_id.eq(token_version.user_id)))
                    .execute(&conn)
                    .map_err(|_| db_error("Error deleting previous user_tokens"))?;

                (token_version.user_id, token_version.version + 1)
            }
            None => {
                let new_user = models::NewUser {
                    google_resource_id: Some(&msg.resource_id),
                    full_name: &msg.full_name,
                    display_name: &msg.display_name,
                };

                let inserted_user: models::User = insert_into(schema::users::table)
                    .values(&new_user)
                    .get_result(&conn)
                    .map_err(|_| db_error("db insert user error"))?;

                (inserted_user.id, 0)
            }
        };

        let new_user_token = models::NewUserToken {
            user_id: user_id,
            google_resource_id: &msg.resource_id,
            access_token: &msg.access_token.access_token,
            refresh_token: &msg.refresh_token,
            token_expiration: &msg.access_token.expires_at,
            version: version,
        };

        let inserted_tokens: models::UserToken = insert_into(schema::user_tokens::table)
            .values(&new_user_token)
            .get_result(&conn)
            .map_err(|_| db_error("db insert user token error"))?;

        Ok(UserIdAndTokenVersion(user_id, inserted_tokens.version))
    }
}

/// Upsert user information, returning the new or existing user's id and version
pub struct GetUserIdAndToken {
    pub resource_id: String,
}

impl Message for GetUserIdAndToken {
    type Result = Result<Option<UserIdAndTokenVersion>>;
}

impl Handler<GetUserIdAndToken> for DbExecutor {
    type Result = Result<Option<UserIdAndTokenVersion>>;

    fn handle(&mut self, msg: GetUserIdAndToken, _: &mut Self::Context) -> Self::Result {
        use views::get_user_token_versions_by_resource;
        let conn = self.0.get().unwrap();

        // Delete previous user tokens if they exist
        get_user_token_versions_by_resource(&conn, &msg.resource_id)
            .map(|o| o.map(|v| UserIdAndTokenVersion(v.user_id, v.version)))
            .map_err(|_| db_error("db select view of user token version error"))
    }
}
