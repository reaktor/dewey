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

use actix::Actor;
use actix_redis::{RedisSessionBackend, RedisActor};
use actix_web::middleware::session::{SessionStorage};
use actix_web::{error, http, Error, FutureResponse, Result};
use actix_web::{middleware, server, App, HttpRequest, HttpResponse};
use futures::future;
use futures::{Future, IntoFuture};

mod upload;

mod db;
use db::DbExecutor;
mod sessions;
use sessions::session_manager;
use sessions::session_manager::{SessionManager};
use sessions::session_routes;
use sessions::session_routes::{is_signed_in_guard, SigninState};


use actix::{Addr, SyncArbiter};

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
            .scope("/login", session_routes::login_scope)
            .resource("/logout", |r| r.f(session_routes::logout_endpoint))
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
