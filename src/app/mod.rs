use askama::Template; // bring trait in scope

use actix_web::middleware::session::{SessionStorage, RequestSession};
use actix_web::{http, Error};
use actix_web::{middleware, server, App, HttpRequest, HttpResponse};
use futures::Future;
use actix::{Actor, SyncArbiter};
use actix_redis::{RedisSessionBackend, RedisActor};

use super::logging;
use super::sessions;
use super::db::DbExecutor;
pub use super::State;

use sessions::session_manager::SessionManager;
use sessions::session_routes::{self, is_signed_in_guard, SigninState};
use sessions::flash::SessionFlash; // enable inserting and applying flash messages to the page

pub mod templates;
mod upload;

use crate::store::ObjectStore;

pub fn start() {
    ::std::env::set_var("RUST_LOG", "actix_web=info,dewey=info");
    logging::init();
    let sys = actix::System::new("dewey");

    // r2d2 pool
    let manager = diesel::r2d2::ConnectionManager::new(dotenv!("DATABASE_URL"));
    let pool = diesel::r2d2::Pool::new(manager).unwrap();

    // Start db executor actors
    let db_addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));
    let redis_addr = RedisActor::start(dotenv!("REDIS_URL"));

    let session_actor = SessionManager {
        redis: redis_addr.clone(),
        pg: db_addr.clone(),
    };

    let session_addr = session_actor.start();

    let store_actor = ObjectStore::new_with_s3_credentials(
        dotenv!("S3_ACCESS_KEY_ID"),
        dotenv!("S3_SECRET_ACCESS_KEY"),
    )
    .expect("No TLS errors starting store_actor");

    let store_addr = store_actor.start();

    use listenfd::ListenFd;
    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(move || {
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
                store: store_addr.clone(),
            })
            .middleware(middleware::Logger::new(r#"%T "%r" %s %b "%{Referer}i""#))
            .middleware(SessionStorage::new(
                RedisSessionBackend::new(dotenv!("REDIS_URL"), &[0; 32])
                    .cookie_secure(true) // cookies require https
                    .cookie_name("sess"),
            ))
            .resource("/example", |r| r.f(upload_example))
            .resource("/upload", |r| {
                r.method(http::Method::POST).with(upload::upload)
            })
            .scope("/login", session_routes::login_scope)
            .resource("/logout", |r| r.f(session_routes::logout_endpoint))
            .resource("/", |r| r.f(index))
            .boxed(),
        ]
    });

    // Autoreload with systemfd & listenfd
    // from: https://actix.rs/docs/autoreload/
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8088").unwrap()
    };

    info!("Started http server: 127.0.0.1:8088");
    info!("                     {}", dotenv!("ROOT_HOST"));
    server.run();
}


fn index(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    use templates::*;
    let req_session = req.session();
    Box::new(
        is_signed_in_guard(req).and_then(move |signin_state: SigninState| {
            let mut page = Page::default();
            req_session.apply_flash(&mut page)?;

            match signin_state {
                SigninState::Valid(ref auth) => page.person(&auth.person),
                SigninState::SignedOutByThirdParty => {
                    page.info("You've been signed out by a third party.")
                }
                SigninState::NotSignedIn => {}
            };
            Ok(HttpResponse::Ok()
                .header(http::header::CONTENT_TYPE, "text/html")
                .body(templates::HelloTemplate { page }.render().unwrap()))
        }),
    )
}

fn upload_example(req: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    use templates::*;
    let req_session = req.session();
    Box::new(
        is_signed_in_guard(req).and_then(move |signin_state: SigninState| {
            let mut page = Page::default();
            req_session.apply_flash(&mut page)?;

            match signin_state {
                SigninState::Valid(ref auth) => page.person(&auth.person),
                SigninState::SignedOutByThirdParty => {
                    page.info("You've been signed out by a third party.")
                }
                SigninState::NotSignedIn => {}
            };

            if page.user_opt.is_none() {
                return Ok(HttpResponse::Found().header("location", "/").finish());
            }

            Ok(HttpResponse::Ok()
                .header(http::header::CONTENT_TYPE, "text/html")
                .body(templates::UploadTemplate { page }.render().unwrap()))
        }),
    )
}
