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

// use diesel::prelude::*;
// use diesel::pg::PgConnection;
// use std::env;

use actix_redis::RedisSessionBackend;
use actix_web::middleware::session::{RequestSession, SessionStorage};
use actix_web::{http, Result};
use actix_web::{middleware, server, App, HttpRequest, HttpResponse};
use actix::Actor;
mod upload;

fn index(req: &HttpRequest<State>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "text/html")
        .body(format!(
            r#"<html>
<head>
    <title>Collect</title>
    <link rel="stylesheet" href="/static/app.css"/>
</head>
<body class="page">
    <h1>Collect</h1>
    {}
</body>
</html>"#,
            if let Some(auth) = req.session().get::<UserAuth>("auth")? {
                format!(
                    "Hello {}<br><a href=\"/logout\">Log out</a>",
                    auth.user_name
                )
            } else {
                "<a href=\"/login\">Log in</a>".to_string()
            }
        )))
}

mod rand_util;

#[derive(Deserialize, Serialize)]
struct UserAuth {
    user_id: i64,
    user_name: String,
    version: i32,
}

fn is_valid(user_auth: &UserAuth) -> bool {
    // TODO: check if version is up to date compared to database
    true
}

fn login(req: &HttpRequest<State>) -> Result<HttpResponse> {
    if let Some(auth) = req.session().get::<UserAuth>("auth")? {
        if is_valid(&auth) {
            Ok(HttpResponse::Found().header("location", "/").finish())
        } else {
            req.session().remove("auth");
            Ok(HttpResponse::Found()
                .header("location", "/login/google?expired=1")
                .finish())
        }
    } else {
        // no login associated with this cookie
        Ok(HttpResponse::Found()
            .header("location", "/login/google")
            .finish())
    }
}

fn login_google(req: &HttpRequest<State>) -> Result<HttpResponse> {
    let redirect_uri = format!("{}/login/google/callback", dotenv!("ROOT_HOST"));
    req.session().remove("auth");

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
mod google_people_client;

fn send_error<T: Debug + Display>(e: T) -> Error {
    error::ErrorInternalServerError(format!("Send error: {}; {:?}", e, e))
}

/// Manually revoke application tokens https://myaccount.google.com/permissions
fn login_google_callback(request: &HttpRequest<State>) -> Result<HttpResponse> {
    if let Some(cause) = request.query().get("error") {
        return Ok(HttpResponse::BadRequest()
            .body(format!("Error during owner authorization: {:?}", cause)));
    }

    let code = match request.query().get("code") {
        None => return Ok(HttpResponse::BadRequest().body("Missing code")),
        Some(code) => code.clone(),
    };

    use google_oauth::ExchangeResult::*;

    let session_mgr: Addr<session_manager::SessionManager> = request.state().sessions.clone();

    match google_oauth::exchange_code_for_token(&code) {
        AccessAndRefreshTokens { access, refresh } => {
            // TODO: Insert refresh tokens and create session
            return Ok(HttpResponse::Found()
                .header("Location", "/?login=refreshed")
                .finish());
        }
        AccessTokenOnly(access) => {
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            // We need this to all be async!
            /*
            session_mgr.send(session_manager::CreateSession {
                access_token: access,
                refresh_token: None,
                ip: request.connection_info().remote().unwrap_or("").to_owned(),
                channel: String::from("web"),
            }).map_err(send_error)
            .and_then(|res: session_manager::CreateSessionResult| {
                use session_manager::CreateSessionResult::*;
                match res {
                    Success(user_session) => {

                    },
                    UserNotFoundNeedsRefreshToken => {
                        // TODO: Create session and link to refresh tokens
                        return Ok(HttpResponse::Found()
                            .header("Location", "/?login=accessonly")
                            .finish());

                    }
                }
            })
            */
        }
        FetchError(error) => {
            error!("Error fetching code: {}", error);
        }
        ParsingError(error) => {
            error!("Error parsing exchange result: {}", error);
        }
        GoogleError(error) => {
            info!("Error logging in user: {}", error);
            return Ok(HttpResponse::Found()
                .header("Location", "/?login=canceled")
                .finish());
        }
    }

    Ok(HttpResponse::Found()
        .header("Location", "/?login=error")
        .finish())
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
use session_manager::{SessionManager, IsValidSession, UpdateUserSession, ValidUserSession};

use actix::{Addr, SyncArbiter};

/// State with DbExecutor address
pub struct State {
    db: Addr<DbExecutor>,
    mem: Addr<RedisActor>,
    sessions: Addr<SessionManager>,
}

mod logging;

fn main() {
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

    println!("Started http server: 127.0.0.1:8088");
    let _ = sys.run();
}
