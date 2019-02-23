//! Redis executor actor
use ::actix::prelude::*;
use actix_web::*;
use std::io;

use actix_redis::*;

use super::db;
use super::db::{DbExecutor, GetUserIdAndToken, UpsertGoogleUser, UserIdAndTokenVersion};
use super::google_oauth;
use super::google_oauth::GoogleAccessToken;
use super::google_people_client::{who_am_i_async, IAm};

use futures;
use futures::future::Either;
use futures::future::Future;
use futures::future::IntoFuture;

use std::fmt::{Debug, Display};

use ::actix::prelude::ResponseFuture;
use actix_redis::Command;
use actix_redis::RespValue;
use actix_web::{error, Error, Result};

/// How often should we recheck that the login is valid?
const SESSION_EXPIRES_IN_MINUTES: i64 = 15;

/// This is session management actor
pub struct SessionManager {
    pub redis: Addr<RedisActor>,
    pub pg: Addr<DbExecutor>,
}

/// Returns the new session's key
pub struct CreateSession {
    /// User's access token
    pub access_token: GoogleAccessToken,
    /// User's refresh token
    pub refresh_token: Option<String>,
    /// Logged in from
    pub ip: String,
    /// "web"
    pub channel: String,
}

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Serialize, Deserialize)]
pub struct ValidUserSession {
    #[serde(rename = "i")]
    user_id: i64,
    #[serde(rename = "v")]
    version: i32,
}

impl Message for CreateSession {
    type Result = Result<CreateSessionResult>;
}

fn send_error<T: Debug + Display>(e: T) -> Error {
    error::ErrorInternalServerError(format!("Send error: {}; {:?}", e, e))
}

fn fetch_error<T: Debug + Display>(e: T) -> Error {
    error::ErrorInternalServerError(format!("Fetch error: {}; {:?}", e, e))
}

pub enum CreateSessionResult {
    Success(ValidUserSession),
    UserNotFoundNeedsRefreshToken,
}

impl From<UserIdAndTokenVersion> for CreateSessionResult {
    fn from(UserIdAndTokenVersion(i, v): UserIdAndTokenVersion) -> Self {
        CreateSessionResult::Success(ValidUserSession {
            user_id: i,
            version: v,
        })
    }
}

impl Handler<CreateSession> for SessionManager {
    type Result = FutureResponse<CreateSessionResult>;

    fn handle(&mut self, msg: CreateSession, _: &mut Self::Context) -> Self::Result {
        let conn = self.pg.clone();
        Box::new(
            // who is this chum trying to create a session?
            // does their account have credentials?
            who_am_i_async(&msg.access_token).and_then(
                move |IAm {
                          resource_name,
                          name,
                      }| {
                    if let Some(refresh_token) = msg.refresh_token {
                        Either::A(
                            conn.send(db::UpsertGoogleUser {
                                resource_id: resource_name.clone(),
                                full_name: name
                                    .clone()
                                    .unwrap_or(format!("No name: {}", resource_name.clone())),
                                display_name: name.unwrap_or(String::from("No name")),
                                access_token: msg.access_token,
                                refresh_token: refresh_token,
                            })
                            .map_err(send_error)
                            .and_then(|res| {
                                res.map_err(|_| {
                                    error::ErrorInternalServerError(
                                        "Error upserting Google User & Token",
                                    )
                                })
                                .map(CreateSessionResult::from)
                            }),
                        )
                    } else {
                        Either::B(
                            conn.send(db::GetUserIdAndToken {
                                resource_id: resource_name,
                            })
                            .map_err(send_error)
                            .and_then(|res| {
                                res.map_err(|_| {
                                    error::ErrorInternalServerError(
                                        "Error getting User Id and Token",
                                    )
                                })
                            })
                            .and_then(move |opt| {
                                match opt {
                                    None => {
                                        // we should revoke your tokens since we did not receive a refresh from you
                                        google_oauth::revoke_token(&msg.access_token)
                                            .map(|_| {
                                                CreateSessionResult::UserNotFoundNeedsRefreshToken
                                            })
                                            .map_err(fetch_error)
                                    }
                                    Some(r) => Ok(CreateSessionResult::from(r)),
                                }
                            }),
                        )
                    }
                },
            ),
        )
    }
}

pub struct IsValidSession(ValidUserSession);

impl Message for IsValidSession {
    type Result = Result<bool, Error>;
}

impl Handler<IsValidSession> for SessionManager {
    type Result = ResponseFuture<bool, Error>;

    fn handle(&mut self, msg: IsValidSession, _: &mut Self::Context) -> Self::Result {
        Box::new(
            self.redis
                .send(Command(resp_array!["GET", format!("ut{}", msg.0.user_id)]))
                .map_err(Error::from)
                .and_then(move |res| match res {
                    Ok(val) => match val {
                        RespValue::Error(err) => Err(error::ErrorInternalServerError(err)),
                        RespValue::Integer(version) => Ok(version == (msg.0.version as i64)),
                        RespValue::SimpleString(s) => {
                            if let Ok(val) = serde_json::from_str::<i32>(&s) {
                                Ok(val == msg.0.version)
                            } else {
                                Ok(false)
                            }
                        }
                        RespValue::BulkString(s) => {
                            if let Ok(val) = serde_json::from_slice::<i32>(&s) {
                                Ok(val == msg.0.version)
                            } else {
                                Ok(false)
                            }
                        }
                        _ => Ok(false),
                    },
                    Err(err) => Err(error::ErrorInternalServerError(err)),
                }),
        )
    }
}

pub struct UpdateUserSession(ValidUserSession);

impl Message for UpdateUserSession {
    type Result = Result<(), Error>;
}

impl Handler<UpdateUserSession> for SessionManager {
    type Result = ResponseFuture<(), Error>;

    fn handle(&mut self, msg: UpdateUserSession, _: &mut Self::Context) -> Self::Result {
        Box::new(
            self.redis
                .send(Command(resp_array![
                    "SET",
                    format!("ut{}", msg.0.user_id),
                    RespValue::Integer(msg.0.version as i64)
                ]))
                .map_err(Error::from)
                .and_then(move |res| match res {
                    Ok(val) => match val {
                        RespValue::Error(err) => Err(error::ErrorInternalServerError(err)),
                        _ => Ok(()),
                    },
                    Err(err) => Err(error::ErrorInternalServerError(err)),
                }),
        )
    }
}

impl Actor for SessionManager {
    type Context = Context<Self>;
}
