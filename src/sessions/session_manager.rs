//! Redis executor actor
use ::actix::prelude::*;
use actix_web::*;
use std::io;

use actix_redis::*;

use super::db;
use super::db::{DbExecutor, UserIdAndTokenVersion};

use super::oauth;
use oauth::google_oauth;
use oauth::google_people_client::{who_am_i_async, IAm};
use oauth::GoogleAccessToken;

use futures;
use futures::future;
use futures::future::{Either, Future};

use std::fmt::{Debug, Display};

use ::actix::prelude::ResponseFuture;
use actix_redis::{Command, RespValue};
use actix_web::{error, Error, Result};

use super::{UserSession, UserSessionKey};

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

impl Message for CreateSession {
    type Result = Result<CreateSessionResult>;
}

fn get_auth_key_and_value(user_id: i64, version: i32) -> (String, String) {
    (format!("ut#{}", user_id), format!("v#{}", version))
}

fn send_error<T: Debug + Display>(e: T) -> Error {
    error::ErrorInternalServerError(format!("Send error: {}; {:?}", e, e))
}

fn fetch_error<T: Debug + Display>(e: T) -> Error {
    error::ErrorInternalServerError(format!("Fetch error: {}; {:?}", e, e))
}

pub enum CreateSessionResult {
    Success(UserSession),
    UserNotFoundNeedsRefreshToken,
}

impl Handler<CreateSession> for SessionManager {
    type Result = FutureResponse<CreateSessionResult>;

    fn handle(&mut self, msg: CreateSession, _: &mut Self::Context) -> Self::Result {
        let conn = self.pg.clone();
        let redis = self.redis.clone();
        Box::new(
            who_am_i_async(&msg.access_token)
                .and_then(
                    move |IAm {
                              resource_name,
                              given_name,
                              display_name,
                              email_address,
                          }| {
                        if let Some(refresh_token) = msg.refresh_token {
                            Either::A(
                                conn.send(db::UpsertGoogleUser {
                                    resource_id: resource_name.clone(),
                                    full_name: display_name.clone(),
                                    display_name: given_name.clone(),
                                    access_token: msg.access_token,
                                    refresh_token: refresh_token,
                                })
                                .map_err(send_error)
                                .and_then(|res| {
                                    res.map_err(|e| {
                                        error!("Error upserting Google User & Token: {:?}", e);
                                        error::ErrorInternalServerError(
                                            "Error upserting Google User & Token",
                                        )
                                    })
                                    .map(
                                        move |UserIdAndTokenVersion(i, v)| {
                                            CreateSessionResult::Success(UserSession {
                                                key: UserSessionKey {
                                                    user_id: i,
                                                    version: v,
                                                },
                                                display_name: display_name,
                                                email_address: email_address,
                                            })
                                        },
                                    )
                                }),
                            )
                        } else {
                            Either::B(
                                conn.send(db::GetUserIdAndToken {
                                    resource_id: resource_name,
                                })
                                .map_err(send_error)
                                .and_then(|res| {
                                    res.map_err(|e| {
                                        error!("Error getting User Id and Token: {:?}", e);
                                        error::ErrorInternalServerError(
                                            "Error getting User Id and Token",
                                        )
                                    })
                                })
                                .and_then(move |opt| {
                                    match opt {
                                        None => {
                                            // we should revoke your tokens since we did not receive a refresh from you
                                            Either::A(google_oauth::revoke_token(&msg.access_token)
                                            .map(|_| {
                                                CreateSessionResult::UserNotFoundNeedsRefreshToken
                                            })
                                            .map_err(fetch_error))
                                        }
                                        Some(UserIdAndTokenVersion(i, v)) => Either::B(future::ok(
                                            CreateSessionResult::Success(UserSession {
                                                key: UserSessionKey {
                                                    user_id: i,
                                                    version: v,
                                                },
                                                display_name: display_name,
                                                email_address: email_address,
                                            }),
                                        )),
                                    }
                                }),
                            )
                        }
                    },
                )
                .and_then(move |create_session_res: CreateSessionResult| {
                    match create_session_res {
                        CreateSessionResult::Success(UserSession {
                            key: ref valid_user_session,
                            ..
                        }) => {
                            // Update redis table
                            // insert into redis
                            info!(
                                "REDIS inserting session into table = {:?}",
                                valid_user_session
                            );
                            let (key, value) = get_auth_key_and_value(
                                valid_user_session.user_id,
                                valid_user_session.version,
                            );
                            Either::A(
                                redis
                                    .send(Command(resp_array!["SET", &key, value]))
                                    .map_err(Error::from)
                                    .and_then(move |res| {
                                        info!("REDIS inserted session into table => {:?}", res);
                                        match res {
                                            Ok(val) => match val {
                                                RespValue::Error(err) => {
                                                    Err(error::ErrorInternalServerError(err))
                                                }
                                                _ => Ok(create_session_res),
                                            },
                                            Err(err) => Err(error::ErrorInternalServerError(err)),
                                        }
                                    }),
                            )
                        }
                        CreateSessionResult::UserNotFoundNeedsRefreshToken => {
                            // Remove from redis table
                            Either::B(future::ok(create_session_res))
                        }
                    }
                }),
        )
    }
}

pub struct IsValidSession(pub UserSessionKey);

impl Message for IsValidSession {
    type Result = Result<bool, Error>;
}

impl Handler<IsValidSession> for SessionManager {
    type Result = ResponseFuture<bool, Error>;

    fn handle(&mut self, msg: IsValidSession, _: &mut Self::Context) -> Self::Result {
        // TODO: Insert expiry information & check if the google account has been signed out
        let (key, expected_value) = get_auth_key_and_value(msg.0.user_id, msg.0.version);
        Box::new(
            self.redis
                .send(Command(resp_array!["GET", key]))
                .map_err(Error::from)
                .and_then(move |res| {
                    info!("REDIS Checking if is valid session = {:?}", res);
                    match res {
                        Ok(val) => match val {
                            RespValue::Error(err) => Err(error::ErrorInternalServerError(err)),
                            RespValue::SimpleString(s) => Ok(expected_value == s),
                            RespValue::BulkString(s) => {
                                Ok(expected_value.as_bytes() == s.as_slice())
                            }
                            _ => Ok(false),
                        },
                        Err(err) => Err(error::ErrorInternalServerError(err)),
                    }
                }),
        )
    }
}

pub struct UpdateUserSession(UserSessionKey);

impl Message for UpdateUserSession {
    type Result = Result<(), Error>;
}

impl Handler<UpdateUserSession> for SessionManager {
    type Result = ResponseFuture<(), Error>;

    fn handle(&mut self, msg: UpdateUserSession, _: &mut Self::Context) -> Self::Result {
        let (key, updated_value) = get_auth_key_and_value(msg.0.user_id, msg.0.version);
        Box::new(
            self.redis
                .send(Command(resp_array!["SET", key, updated_value]))
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
