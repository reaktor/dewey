//! Db executor actor
use ::actix::prelude::*;
use actix_web::{error, Error, Result};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

use crate::sessions::GoogleAccessToken;

mod models;
mod schema;
mod views;

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSessionKey {
    #[serde(rename = "i")]
    pub user_id: i64,
    #[serde(rename = "v")]
    pub version: i32,
}

/// User in the database
pub enum User {
    Person(PersonUser),
    Reserved(ReservedUser),
    Plugin(PluginUser),
}

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonUser {
    #[serde(rename = "i")]
    pub user_id: i64,
    #[serde(rename = "pe")]
    pub public_email: String,
    #[serde(rename = "dn")]
    pub display_name: String,
    #[serde(rename = "fn")]
    pub full_name: String,
}

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReservedUser {
    #[serde(rename = "i")]
    user_id: i64,
    #[serde(rename = "dn")]
    display_name: String,
    #[serde(rename = "fn")]
    full_name: String,
}

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginUser {
    #[serde(rename = "i")]
    user_id: i64,
    #[serde(rename = "dn")]
    display_name: String,
    #[serde(rename = "fn")]
    full_name: String,
}

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

fn db_error<T: Into<String>, U: std::fmt::Debug>(message: T, err: U) -> Error {
    let mstr = message.into();
    error!("db_error: {}; {:?}", mstr, err);
    std::io::Error::new(std::io::ErrorKind::Other, mstr).into()
}

/// Upsert user information, returning the new or existing user's id and version
pub struct UpsertGoogleUser {
    pub resource_id: String,
    pub full_name: String,
    pub display_name: String,
    pub email_address: String, // TODO: add to database
    pub access_token: GoogleAccessToken,
    pub refresh_token: String,
}

impl Message for UpsertGoogleUser {
    type Result = Result<(PersonUser, UserSessionKey)>;
}

impl Handler<UpsertGoogleUser> for DbExecutor {
    type Result = Result<(PersonUser, UserSessionKey)>;

    fn handle(&mut self, msg: UpsertGoogleUser, _: &mut Self::Context) -> Self::Result {
        use views::{get_user_token_versions_by_resource, ViewUserIdTokenVersion};
        let conn = self.0.get().unwrap();

        // Delete previous user tokens if they exist
        let user_exists: Option<ViewUserIdTokenVersion> =
            get_user_token_versions_by_resource(&conn, &msg.resource_id)
                .map_err(|e| db_error("db select view of user token version error", e))?;

        let (user_id, version): (i64, i32) = match user_exists {
            Some(token_version) => {
                // User exists
                use schema::user_tokens::dsl::*;
                // Delete previous user tokens
                diesel::delete(user_tokens.filter(user_id.eq(token_version.user_id)))
                    .execute(&conn)
                    .map_err(|e| db_error("Error deleting previous user_tokens", e))?;

                (token_version.user_id, token_version.version + 1)
            }
            None => {
                let person_kind = schema::UserKind::Person;
                let new_user = models::NewUser {
                    google_resource_id: Some(&msg.resource_id),
                    full_name: &msg.full_name,
                    display_name: &msg.display_name,
                    public_email: Some(&msg.email_address),
                    kind: &person_kind,
                };

                let inserted_user: models::User = insert_into(schema::users::table)
                    .values(&new_user)
                    .get_result(&conn)
                    .map_err(|e| db_error("db insert user error", e))?;

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
            .map_err(|e| db_error("db insert user token error", e))?;

        Ok((
            PersonUser {
                display_name: msg.display_name,
                user_id: user_id.clone(),
                full_name: msg.full_name,
                public_email: msg.email_address,
            },
            UserSessionKey {
                user_id,
                version: inserted_tokens.version,
            },
        ))
    }
}

/// Upsert user information, returning the new or existing user's id and version
pub struct GetUserByResourceId {
    pub resource_id: String,
}

impl Message for GetUserByResourceId {
    type Result = Result<Option<(PersonUser, UserSessionKey)>>;
}

/// errors if the record does not exist
fn get_user_by_id(conn: &PgConnection, user_id: i64) -> Result<User> {
    {
        use schema::users::dsl::*;
        users
            .filter(id.eq(user_id))
            .get_result::<models::User>(conn)
            .map_err(|e| db_error("db select get user by id error", e))
            .and_then(User::try_from_model)
    }
}

impl Handler<GetUserByResourceId> for DbExecutor {
    type Result = Result<Option<(PersonUser, UserSessionKey)>>;

    fn handle(&mut self, msg: GetUserByResourceId, _: &mut Self::Context) -> Self::Result {
        use views::get_user_token_versions_by_resource;
        let conn = self.0.get().unwrap();

        // Delete previous user tokens if they exist
        get_user_token_versions_by_resource(&conn, &msg.resource_id)
            .map_err(|e| db_error("db select view of user token version error", e))
            .and_then(move |o| match o {
                Some(v) => {
                    get_user_by_id(&conn, v.user_id).and_then(|db_user: User| match db_user {
                        User::Person(person_user) => Ok(Some((
                            person_user,
                            UserSessionKey {
                                user_id: v.user_id,
                                version: v.version,
                            },
                        ))),
                        User::Reserved(r) => Err(db_error(
                            "GetUserByResourceId: Unexpected reserved user with google resource",
                            r,
                        )),
                        User::Plugin(r) => Err(db_error(
                            "GetUserByResourceId: Unexpected plugin user with google resource",
                            r,
                        )),
                    })
                }
                None => Ok(None),
            })
    }
}

impl User {
    fn try_from_model(user_model: models::User) -> Result<Self> {
        let user_id = user_model.id;
        Ok(match user_model.kind {
            schema::UserKind::Person => User::Person(PersonUser {
                user_id: user_model.id,
                public_email: user_model.public_email.ok_or_else(move || {
                    db_error("Expected person user to have public_email", user_id)
                })?,
                display_name: user_model.display_name,
                full_name: user_model.full_name,
            }),
            schema::UserKind::Reserved => User::Reserved(ReservedUser {
                user_id: user_model.id,
                display_name: user_model.display_name,
                full_name: user_model.full_name,
            }),
            schema::UserKind::Plugin => User::Plugin(PluginUser {
                user_id: user_model.id,
                display_name: user_model.display_name,
                full_name: user_model.full_name,
            }),
        })
    }
}
