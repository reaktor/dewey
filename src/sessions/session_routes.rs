//! Redis executor actor
use ::actix::prelude::*;
use actix_web::*;

use super::oauth;
use oauth::google_oauth;

use futures;
use futures::future;
use futures::future::{Either, Future, IntoFuture};

use std::fmt::{Debug, Display};

use actix_web::middleware::session::RequestSession;
use actix_web::{error, Error, Result};

use super::flash::SessionFlash;
use super::session_manager::{self, SessionManager};
use super::UserSession;

// Main application state
use super::super::State;

const USER_SESSION_KEY: &'static str = "user_session";

fn is_valid(
    user_session: &UserSession,
    session_mgr: &Addr<SessionManager>,
) -> impl Future<Item = bool, Error = Error> {
    session_mgr
        .send(session_manager::IsValidSession(user_session.key.clone()))
        .flatten()
}

pub enum SigninState {
    Valid(UserSession),
    SignedOutByThirdParty,
    NotSignedIn,
}

pub fn is_signed_in_guard(
    req: &HttpRequest<State>,
) -> impl Future<Item = SigninState, Error = Error> {
    let req_session = req.session();
    let session_mgr: Addr<SessionManager> = req.state().sessions.clone();

    req_session
        .get::<UserSession>(USER_SESSION_KEY)
        .into_future()
        .and_then(move |auth_opt| {
            info!("User request's auth = {:?}", auth_opt);
            if let Some(auth) = auth_opt {
                Either::A(is_valid(&auth, &session_mgr).and_then(
                    move |sign_in_valid: bool| {
                        info!(
                            "Checking if session is auth: {:?}; {:?}",
                            auth, sign_in_valid
                        );
                        if sign_in_valid {
                            Ok(SigninState::Valid(auth))
                        } else {
                            req_session
                                .flash("You've been signed out by another location.")?;
                            req_session.remove(USER_SESSION_KEY);
                            Ok(SigninState::SignedOutByThirdParty)
                        }
                    },
                ))
            } else {
                // no login associated with this cookie
                Either::B(future::ok(SigninState::NotSignedIn))
            }
        })
}

fn login(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let req_session = req.session();

    Box::new(
        is_signed_in_guard(req).and_then(move |signin_state: SigninState| {
            match signin_state {
                SigninState::Valid(_) => {
                    req_session.flash("You're signed in!")?;
                    Ok(HttpResponse::Found().header("location", "/").finish())
                }
                SigninState::SignedOutByThirdParty => Ok(HttpResponse::Found()
                    .header("location", "/login/google?expired=1")
                    .finish()),
                SigninState::NotSignedIn => {
                    // no login associated with this cookie
                    Ok(HttpResponse::Found()
                        .header("location", "/login/google")
                        .finish())
                }
            }
        }),
    )
}

fn login_google(_req: &HttpRequest<State>) -> Result<HttpResponse> {
    let redirect_uri = format!("{}/login/google/callback", dotenv!("ROOT_HOST"));

    // TODO: Think about using state to transfer login across browsers like events-nyc does
    // This essentially requires us to set the state directly into Redis, and may require us
    // to be able to read and overwrite our RedisCookieStorage backend
    // let state = rand_util::random_string(8);
    // req.session().set("auth-state", &state)?;

    Ok(HttpResponse::Found()
        .header(
            "location",
            get_redirect_url(&redirect_uri, None, Some("reaktor.fi")),
        )
        .finish())
}

fn send_error<T: Debug + Display>(e: T) -> Error {
    error::ErrorInternalServerError(format!("Send error: {}; {:?}", e, e))
}

/// Manually revoke application tokens https://myaccount.google.com/permissions
fn login_google_callback(
    request: &HttpRequest<State>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    if let Some(cause) = request.query().get("error") {
        return Box::new(future::ok(
            HttpResponse::BadRequest()
                .body(format!("Error during owner authorization: {:?}", cause)),
        ));
    }

    let code = match request.query().get("code") {
        None => return Box::new(future::ok(HttpResponse::BadRequest().body("Missing code"))),
        Some(code) => code.clone(),
    };

    let session_mgr: Addr<session_manager::SessionManager> = request.state().sessions.clone();
    let req_session = request.session();

    let conn_info = request.connection_info().remote().unwrap_or("").to_owned();
    info!(
        "login_google_callback exchange_code_for_token: {}",
        &conn_info
    );
    Box::new(
        google_oauth::exchange_code_for_token(&code).and_then(move |result| {
            info!("login_google_callback exchange_code_for_token result");
            use google_oauth::ExchangeResult::*;
            let create_session = match result {
                AccessAndRefreshTokens { access, refresh } => {
                    info!("Received Access & Refresh Tokens");
                    session_manager::CreateSession {
                        access_token: access,
                        refresh_token: Some(refresh),
                        ip: conn_info,
                        channel: String::from("web"),
                    }
                }
                AccessTokenOnly(access) => {
                    info!("Received only Access Token");
                    session_manager::CreateSession {
                        access_token: access,
                        refresh_token: None,
                        ip: conn_info,
                        channel: String::from("web"),
                    }
                }
            };
            let is_new_account = create_session.refresh_token.is_some();
            // We need this to all be async!
            session_mgr
                .send(create_session)
                .map_err(send_error)
                .and_then(move |res: Result<session_manager::CreateSessionResult>| {
                    res.and_then(|create_result: session_manager::CreateSessionResult| {
                        use session_manager::CreateSessionResult::*;
                        match create_result {
                            Success(user_session) => {
                                match req_session.set(USER_SESSION_KEY, user_session) {
                                    Ok(_) => Ok(HttpResponse::Found()
                                        .header(
                                            "Location",
                                            if is_new_account {
                                                req_session.flash("You've signed up!")?;
                                                "/?login=signed-up"
                                            } else {
                                                req_session.flash("You've signed in!")?;
                                                "/?login=signed-in"
                                            },
                                        )
                                        .finish()),
                                    Err(e) => {
                                        req_session.flash(
                                            "An unexpected error occurred while signing you in.",
                                        )?;
                                        warn!("Error setting user session {:?}", e);
                                        Ok(HttpResponse::Found()
                                            .header("Location", "/?login=session-failure")
                                            .finish())
                                    }
                                }
                            }
                            UserNotFoundNeedsRefreshToken => {
                                req_session
                                    .flash("You may have been signed out by another location. Please try signing in again.")?;
                                Ok(HttpResponse::Found()
                                    .header("Location", "/?login=accessonly+revoked")
                                    .finish())
                            }
                        }
                    })
                })
        }),
    )
}

pub fn get_redirect_url(redirect_uri: &str, state: Option<&str>, domain: Option<&str>) -> String {
    let gapi_client_id = dotenv!("GOOGLE_OAUTH_CLIENT_ID");

    let oauth_endpoint = "https://accounts.google.com/o/oauth2/v2/auth";
    // let calendar_scope = "https://www.googleapis.com/auth/calendar";
    // let emails_readonly_scope = "https://www.googleapis.com/auth/user.emails.read";
    let profile_scope = "https://www.googleapis.com/auth/userinfo.profile";
    let scopes = format!("{}", profile_scope);

    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&access_type=offline&state={}&hd={}&prompt=select_account",
        oauth_endpoint, gapi_client_id, redirect_uri, scopes, state.unwrap_or(""), domain.unwrap_or("")
    )
}

pub fn logout_endpoint(req: &HttpRequest<State>) -> Result<HttpResponse> {
    let req_session = req.session();
    req_session.remove(USER_SESSION_KEY);
    req_session.flash("You have signed out.")?;

    Ok(HttpResponse::Found().header("location", "/").finish())
}

pub fn login_scope(scope: actix_web::Scope<State>) -> actix_web::Scope<State> {
    scope
        .resource("", |r| r.f(login))
        .resource("/google/callback", |r| r.f(login_google_callback))
        .resource("/google", |r| r.f(login_google))
}
