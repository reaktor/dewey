#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
extern crate actix_web;
extern crate dotenv;

// use diesel::prelude::*;
// use diesel::pg::PgConnection;
// use dotenv::dotenv;
// use std::env;

// Need to fix schema enum types and then we can enable it
// pub mod schema;
pub mod models;

use actix_web::{http, server, App, HttpRequest, HttpResponse};

mod upload;

fn index(_req: &HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "text/html")
        .body(r#"<html>
<head>
    <title>Collect</title>
    <link rel="stylesheet" href="/static/app.css"/>
</head>
<body class="page">
    <h1>Collect</h1>
    <a href="/static/app.html">Upload</a>
</body>
</html>"#)
}

fn index2(_req: &HttpRequest) -> &'static str {
    println!("Hello world2---");
    "Hello world 2!"
}

fn main() {
    server::new(|| {
        vec![
            App::new().prefix("/static").handler(
                "/",
                actix_web::fs::StaticFiles::new("./static")
                    .unwrap()
                    .show_files_listing(),
            ),
            App::new()
                .resource("/app", |r| r.f(index2))
                .resource("/upload", |r| r.method(http::Method::POST).with(upload::upload))
                .resource("/", |r| r.f(index)),
        ]
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run();
}
