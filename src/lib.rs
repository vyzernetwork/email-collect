#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate sendgrid;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use sendgrid::SGClient;
use sendgrid::{Destination, Mail};

pub mod schema;
pub mod models;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

use models::{EmailAddress, NewEmailAddress};

pub fn save_email<'a>(conn: &PgConnection, address: &'a str) -> EmailAddress {
    use schema::emails;

    let new_email = NewEmailAddress {
        address,
    };

    diesel::insert_into(emails::table)
        .values(&new_email)
        .get_result(conn)
        .expect("Error saving new email address.")
}

pub fn send_confirmation<'a>(conn: &PgConnection, address: &'a str) {
    use schema::emails;

    dotenv().ok();

    let api_key = env::var("SENDGRID_API_KEY")
        .expect("SENDGRID_API_KEY must be set.");

    let sg = SGClient::new(api_key);

    let mut x_smtpapi = String::new();
    x_smtpapi.push_str(r#"{"unique_args":{"test":7}}"#);

    let mail_info = Mail::new()
        .add_to(Destination {
            address,
            name: address,
        })
        .add_from("noreply@vyzer.network")
        .add_subject("Please confirm your email")
        .add_html("
            <h1>Vyzer Network</h1>
            <h4>Thanks for signing up! We will only send you the most important updates on our launch plans and sale announcements.</h4>
            <br/>
            <p>https://vyzer.network</p>
        ")
        .add_from_name("Vyzer Network")
        .add_header("x-cool".to_string(), "indeed")
        .add_x_smtpapi(&x_smtpapi);


    match sg.send(mail_info) {
        Err(err) => println!("Error: {}", err),
        Ok(body) => println!("Response: {}", body),
    };
}
