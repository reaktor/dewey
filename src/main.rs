#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate redis_async;
extern crate actix_web;
extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;
#[macro_use]
extern crate log;

extern crate openssl_probe;

// use diesel::prelude::*;
// use diesel::pg::PgConnection;
// use std::env;

use std::fmt::{Debug, Display};

use actix::Actor;
use actix_redis::RedisSessionBackend;
use actix_web::middleware::session::{RequestSession, SessionStorage};
use actix_web::{error, http, Error, FutureResponse, Result};
use actix_web::{middleware, server, App, HttpRequest, HttpResponse};
mod upload;
use futures::future;
use futures::future::Future;

fn index(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        is_signed_in_guard(req).and_then(move |signin_state: SigninState| {
            Ok(HttpResponse::Ok()
                .header(http::header::CONTENT_TYPE, "text/html")
                .body(format!(
                    r#"<html>
<head>
    <title>Collect</title>
    <link rel="stylesheet" href="/static/app.css?3"/>
</head>
<body class="page">
    <h1>Collect</h1>
    {}
</body>
</html>"#,
                    match signin_state {
                        SigninState::Valid(auth) => format!(
                            "You're logged in<br>{}<br><a href=\"/logout\">Log out</a>",
                            serde_json::to_string(&auth).unwrap()
                        ),
                        SigninState::SignedOutByThirdParty => {
                            "<div class=\"flash flash-info\">Your account has been signed out by another location</div><br><a href=\"/login\">Log in</a>".to_string()
                        }
                        SigninState::NotSignedIn => "<a href=\"/login\">Log in</a>".to_string(),
                    }
                )))
        }),
    )
}

mod rand_util;

fn is_valid(
    user_session: &ValidUserSession,
    session_mgr: &Addr<SessionManager>,
) -> impl Future<Item = bool, Error = Error> {
    session_mgr
        .send(session_manager::IsValidSession(user_session.clone()))
        .flatten()
}

enum SigninState {
    Valid(ValidUserSession),
    SignedOutByThirdParty,
    NotSignedIn,
}

use futures::IntoFuture;

fn is_signed_in_guard(req: &HttpRequest<State>) -> impl Future<Item = SigninState, Error = Error> {
    let req_session = req.session();
    let session_mgr: Addr<SessionManager> = req.state().sessions.clone();

    req_session
        .get::<ValidUserSession>("auth")
        .into_future()
        .and_then(move |auth_opt| {
            info!("User request's auth = {:?}", auth_opt);
            if let Some(auth) = auth_opt {
                future::Either::A(is_valid(&auth, &session_mgr).and_then(
                    move |sign_in_valid: bool| {
                        info!("Checking if session is auth: {:?}; {:?}", auth, sign_in_valid);
                        if sign_in_valid {
                            future::ok(SigninState::Valid(auth))
                        } else {
                            req_session.remove("auth");
                            future::ok(SigninState::SignedOutByThirdParty)
                        }
                    },
                ))
            } else {
                // no login associated with this cookie
                future::Either::B(future::ok(SigninState::NotSignedIn))
            }
        })
}

fn login(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let req_session = req.session();

    Box::new(is_signed_in_guard(req).map(|signin_state: SigninState| {
        match signin_state {
            SigninState::Valid(_) => HttpResponse::Found().header("location", "/").finish(),
            SigninState::SignedOutByThirdParty => HttpResponse::Found()
                .header("location", "/login/google?expired=1")
                .finish(),
            SigninState::NotSignedIn => {
                // no login associated with this cookie
                HttpResponse::Found()
                    .header("location", "/login/google")
                    .finish()
            }
        }
    }))
}

fn login_google(req: &HttpRequest<State>) -> Result<HttpResponse> {
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

mod google_oauth;
mod google_oauth_async;
mod google_people_client;

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

    use google_oauth_async::ExchangeResult::*;

    let session_mgr: Addr<session_manager::SessionManager> = request.state().sessions.clone();
    let req_session = request.session();

    let conn_info = request.connection_info().remote().unwrap_or("").to_owned();
    info!(
        "login_google_callback exchange_code_for_token: {}",
        &conn_info
    );
    Box::new(
        google_oauth_async::exchange_code_for_token(&code).and_then(move |result| {
            info!("login_google_callback exchange_code_for_token result");
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
                    res.map(|create_result: session_manager::CreateSessionResult| {
                        use session_manager::CreateSessionResult::*;
                        match create_result {
                            Success(user_session) => match req_session.set("auth", user_session) {
                                Ok(_) => HttpResponse::Found()
                                    .header(
                                        "Location",
                                        if is_new_account {
                                            "/?login=signed-up"
                                        } else {
                                            "/?login=signed-in"
                                        },
                                    )
                                    .finish(),
                                Err(e) => {
                                    warn!("Error setting user session {:?}", e);
                                    HttpResponse::Found()
                                        .header("Location", "/?login=session-failure")
                                        .finish()
                                }
                            },
                            UserNotFoundNeedsRefreshToken => {
                                // TODO: Create session and link to refresh tokens
                                HttpResponse::Found()
                                    .header("Location", "/?login=accessonly+revoked")
                                    .finish()
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

fn logout(req: &HttpRequest<State>) -> Result<HttpResponse> {
    req.session().remove("auth");

    Ok(HttpResponse::Found().header("location", "/").finish())
}

use actix_redis::RedisActor;
mod db;
use db::DbExecutor;
mod session_manager;
use session_manager::{IsValidSession, SessionManager, UpdateUserSession, ValidUserSession};

use actix::{Addr, SyncArbiter};

/// State with DbExecutor address
pub struct State {
    db: Addr<DbExecutor>,
    mem: Addr<RedisActor>,
    sessions: Addr<SessionManager>,
}

mod logging;

fn main() {
    // pulled in for any weird errors that may happen with openssl timeouts
    openssl_probe::init_ssl_cert_env_vars();
    ::std::env::set_var("RUST_LOG", "actix_web=info,dewey=info");
    logging::init();
    let sys = actix::System::new("dewey");

    // r2d2 pool
    let manager = diesel::r2d2::ConnectionManager::new(dotenv!("DATABASE_URL"));
    let pool = diesel::r2d2::Pool::new(manager).unwrap();

    // Start db executor actors
    let db_addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));
    let redis_addr = actix_redis::RedisActor::start(dotenv!("REDIS_URL"));

    let session_actor = SessionManager {
        redis: redis_addr.clone(),
        pg: db_addr.clone(),
    };

    let session_addr = session_actor.start();

    server::new(move || {
        vec![
            App::new()
                .prefix("/static")
                .handler(
                    "/",
                    actix_web::fs::StaticFiles::new("./static")
                        .unwrap()
                        .show_files_listing(),
                )
                .boxed(),
            App::with_state(State {
                db: db_addr.clone(),
                mem: redis_addr.clone(),
                sessions: session_addr.clone(),
            })
            .middleware(middleware::Logger::new(r#"%T "%r" %s %b "%{Referer}i""#))
            .middleware(SessionStorage::new(
                RedisSessionBackend::new(dotenv!("REDIS_URL"), &[0; 32])
                    .cookie_secure(true) // cookies require https
                    .cookie_name("sess"),
            ))
            .resource("/upload", |r| {
                r.method(http::Method::POST).with(upload::upload)
            })
            .scope("/login", |scope| {
                scope
                    .resource("", |r| r.f(login))
                    .resource("/google/callback", |r| r.f(login_google_callback))
                    .resource("/google", |r| r.f(login_google))
            })
            .resource("/logout", |r| r.f(logout))
            .resource("/", |r| r.f(index))
            .boxed(),
        ]
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .start();

    info!("Started http server: 127.0.0.1:8088");
    info!("                     {}", dotenv!("ROOT_HOST"));
    let _ = sys.run();
}
