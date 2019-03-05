use askama::Template; // bring trait in scope

use actix_redis::RedisSessionBackend;
use actix_web::middleware::session::{RequestSession, SessionStorage};
use actix_web::{http, Error};
use actix_web::{middleware, server, App, HttpRequest, HttpResponse};
use futures::Future;

use super::logging;
use super::sessions;
pub use super::State;

use sessions::flash::SessionFlash;
use sessions::session_routes::{self, is_signed_in_guard, SigninState}; // enable inserting and applying flash messages to the page

pub mod templates;
mod upload;

use crate::store::ObjectStore;

pub fn start(state: State) {
    ::std::env::set_var("RUST_LOG", "actix_web=info,dewey=info");
    logging::init();

    let host = String::from(state.config.http_host());
    let port = state.config.http_port();
    let redis_url = String::from(state.config.redis_url());

    use listenfd::ListenFd;
    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(move || {
        vec![
            App::new()
                .prefix("/static")
                .handler("/", actix_web::fs::StaticFiles::new("./static").unwrap())
                .boxed(),
            App::with_state(state.clone())
                .middleware(middleware::Logger::new(r#"%T "%r" %s %b "%{Referer}i""#))
                .middleware(SessionStorage::new(
                    RedisSessionBackend::new(redis_url.clone(), &[0; 32])
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
        server.bind(format!("127.0.0.1:{}", port)).unwrap()
    };

    info!("Started http server: {}:{}", host, port);
    info!("                     https://{}:{}", host, port);
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
