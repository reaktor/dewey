#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_newtype;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate redis_async;
extern crate actix;
extern crate actix_web;
extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;
#[macro_use]
extern crate log;
extern crate askama; // for the Template trait and custom derive macro

mod app;
pub mod config;
mod db;
pub mod object;
pub mod property;
mod sessions;
pub mod state;
pub mod user;

use sessions::flash::SessionFlash;
use sessions::session_manager::SessionManager;

// enable inserting and applying flash messages to the page
use sessions::session_routes::{is_signed_in_guard, SigninState};

use clap::{App, AppSettings, Arg, SubCommand};

use self::store::ObjectStore;
pub use object::store;

pub use config::Configuration;
pub use state::State;

mod logging;

fn main() {
    if let Err(dotenv_error) = dotenv::dotenv() {
        warn!("Unable to process the .env file: {}", dotenv_error);
    }
    let config = Configuration::new().from_environment();

    let args = App::new("Dewey Collect")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("File collection and organization")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("DEBUG")
                .short("d")
                .long("debug")
                .help("Verbose output for troubleshooting"),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("Starts the Dewey Collect Web Application Server")
                .arg(
                    Arg::with_name("PORT")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .help(&format!(
                            "Specify the port to start on [default: {}]",
                            config.http_port()
                        )),
                )
                .arg(
                    Arg::with_name("HOST")
                        .short("h")
                        .long("host")
                        .value_name("HOST")
                        .help(&format!(
                            "Specify the public hostname for the server [default: {}]",
                            config.http_host()
                        )),
                ),
        )
        .subcommand(
            SubCommand::with_name("sessions").about("Displays information about active sessions"),
        )
        .get_matches();

    match args.subcommand_name() {
        Some("start") => {
            let _sys = actix::System::new("dewey");
            let start_args = args.subcommand_matches("start").unwrap();
            match config.from_arguments(start_args) {
                Ok(config) => {
                    println!("Configuration: {:?}", config);
                    app::start(State::new(&config))
                }
                Err(error) => println!("\nInvalid server configuration: {}\n", error),
            };
        }
        Some("sessions") => {
            unimplemented!("Sessions not yet implemented");
        }
        _ => {}
    }
}
