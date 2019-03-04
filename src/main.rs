#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_newtype;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate derive_error;
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
extern crate askama; // for the Template trait and custom derive macro

pub mod object;
pub mod property;
pub mod user;
mod app;

use actix::Addr;
use actix_redis::RedisActor;

mod db;
use db::DbExecutor;
use clap::{App, Arg, SubCommand};

mod sessions;
use sessions::flash::SessionFlash;
use sessions::session_manager::SessionManager;
use sessions::session_routes::{self, is_signed_in_guard, SigninState}; // enable inserting and applying flash messages to the page

pub use object::store;
use self::store::ObjectStore;

/// State with DbExecutor address
pub struct State {
    db: Addr<DbExecutor>,
    mem: Addr<RedisActor>,
    sessions: Addr<SessionManager>,
    store: Addr<ObjectStore>,
}

mod logging;

fn main() {
    let args = App::new("Dewey Collect")
        .version(dotenv!("CARGO_PKG_VERSION"))
        .author(dotenv!("CARGO_PKG_AUTHORS"))
        .about("File collection and organization")
        .arg(Arg::with_name("debug")
             .short("d")
             .help("Verbose output for troubleshooting"))
        .subcommand(SubCommand::with_name("start")
                    .about("Starts the Dewey Collect Web Application Server")
                    .arg(Arg::with_name("port")
                         .short("p")
                         .value_name("PORT")
                         .help("Specify the port to start on [default: 8088]")))
        .get_matches();

    match args.subcommand_name() {
        Some("start") => app::start(),
        _ => {}
    }
}
