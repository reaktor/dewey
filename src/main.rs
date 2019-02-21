#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate actix_web;
extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;
#[macro_use]
extern crate log;

// use diesel::prelude::*;
// use diesel::pg::PgConnection;
// use std::env;

// Need to fix schema enum types and then we can enable it
// pub mod schema;
pub mod models;

use actix_redis::RedisSessionBackend;
use actix_web::middleware::session::{RequestSession, SessionStorage};
use actix_web::{http, Result};
use actix_web::{middleware, server, App, HttpRequest, HttpResponse};

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

    match google_oauth::exchange_code_for_token(&code) {
        AccessAndRefreshTokens { access, refresh } => {
            // TODO: Insert refresh tokens and create session
            return Ok(HttpResponse::Found().header("Location", "/?login=refreshed").finish())
        },
        AccessTokenOnly(access) => {
            // TODO: Create session and link to refresh tokens
            return Ok(HttpResponse::Found().header("Location", "/?login=accessonly").finish())
        },
        FetchError(error) => {
            error!("Error fetching code: {}", error);
        },
        ParsingError(error) => {
            error!("Error parsing exchange result: {}", error);
        },
        GoogleError(error) => {
            info!("Error logging in user: {}", error);
            return Ok(HttpResponse::Found().header("Location", "/?login=canceled").finish())
        }
    }

    Ok(HttpResponse::Found().header("Location", "/?login=error").finish())
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

mod db;
use db::{CreateUser, DbExecutor};

use actix::{Addr, SyncArbiter};

/// State with DbExecutor address
pub struct State {
    db: Addr<DbExecutor>,
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
    let addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));

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
            App::with_state(State { db: addr.clone() })
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
