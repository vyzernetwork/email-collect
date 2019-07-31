#![feature(proc_macro_hygiene, decl_macro)]

extern crate diesel;
extern crate email_collect;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate rocket_cors;
#[macro_use] extern crate serde_derive;
#[macro_use(debug)] extern crate slog;
extern crate rocket_slog;
extern crate sloggers;

use diesel::prelude::*;
use email_collect::{establish_connection, save_email, send_confirmation};
use rocket::response::status;
use rocket::request::{Form, FromForm, LenientForm};
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::fs::OpenOptions;
use std::io::prelude::*;

use std::error::Error;
use rocket::config::{Config, Environment, LoggingLevel};
use rocket_slog::{SlogFairing, SyncLogger};
use sloggers::{
    Build,
    terminal::{
        TerminalLoggerBuilder,
        Destination,
    },
    types::Severity,
};

#[get("/")]
fn index(log: SyncLogger) -> &'static str {
    debug!(log, "triggered");
    "Hello, world!"
}

#[derive(Deserialize, Debug)]
struct Email {
    address: String,
}

use email_collect::models::{EmailAddress, NewEmailAddress};

#[post("/email", format="json", data = "<email>")]
fn collect_email(log: SyncLogger, email: Json<Email>) -> status::Accepted<String> {
    use email_collect::schema::emails::dsl::{emails, confirmed};

    // First just write this to a file.
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("emails")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", email.address) {
        eprintln!("Couldn't write to file: {}", e);
    }

    // Connect to the Postgres DB.
    let connection = establish_connection();

    let found = emails.find(email.address.as_str()).load::<EmailAddress>(&connection).unwrap();

    if found.is_empty() {
        let ret_email = save_email(&connection, email.address.as_str());
        debug!(log, "{:?}", ret_email);

        debug!(log, "Sending confirmation email...");
        send_confirmation(&connection, email.address.as_str());

        return status::Accepted(Some(format!("Saved new email: {:?}", ret_email.address)));
    }

    if !found[0].confirmed {

        return status::Accepted(Some(format!("Sent new confirmation to email: {:?}", found[0].address)));
    }

    debug!(log, "{:?}", email);
    debug!(log, "{:?}", found.is_empty());

    status::Accepted(Some(format!("Email address has already been saved and confirmed.")))
}

fn main() -> Result<(), Box<Error>> {
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stderr);
    let logger = builder.build()?;
    let slogging = SlogFairing::new(logger);

    let config = Config::build(Environment::Development)
        .log_level(LoggingLevel::Off) // disables logging
        .finalize()
        .unwrap();


    let cors = rocket_cors::CorsOptions {
        send_wildcard: true,
        ..Default::default()
    }.to_cors()?;

    rocket::custom(config)
        .mount("/", routes![index, collect_email])
        .attach(cors)
        .attach(slogging)
        .launch();

    Ok(())
}
