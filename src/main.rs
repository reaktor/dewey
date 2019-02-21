#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
extern crate actix_web;
extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;

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

fn index(req: &HttpRequest) -> Result<HttpResponse> {
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
            if let Some(id) = req.session().get::<String>("name")? {
                format!("Hello {}<br><a href=\"/logout\">Log out</a>", id)
            } else {
                "<a href=\"/login\">Log in</a>".to_string()
            }
        )))
}

mod rand_util;

fn login(req: &HttpRequest) -> Result<HttpResponse> {
    req.session().set(
        "name",
        format!("Cole Lawrence {}", rand_util::random_string(4)),
    )?;
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, format!("{}/", dotenv!("ROOT_HOST")))
        .finish())
}

fn logout(req: &HttpRequest) -> Result<HttpResponse> {
    req.session().remove("name");

    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, format!("{}/", dotenv!("ROOT_HOST")))
        .finish())
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("dewey");

    server::new(|| {
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
            App::new()
                .middleware(middleware::Logger::new(r#"%T "%r" %s %b "%{Referer}i""#))
                .middleware(SessionStorage::new(
                    RedisSessionBackend::new(dotenv!("REDIS_URL"), &[0; 32])
                        .cookie_secure(true) // cookies require https
                        .cookie_name("sess")
                ))
                .resource("/upload", |r| {
                    r.method(http::Method::POST).with(upload::upload)
                })
                .resource("/login", |r| r.f(login))
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
