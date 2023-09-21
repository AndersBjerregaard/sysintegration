#[macro_use] extern crate rocket;

use std::collections::{hash_set, HashSet};

use amiquip::{Connection, Exchange, Publish, Result, ExchangeDeclareOptions};
use rocket::fairing::{Fairing, Kind, Info};
use rocket::http::{Header, Status};
use rocket::{Request, Response};
use rocket::{form::Form, fairing::AdHoc};
use rocket::response::{content, status};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket_cors::{AllowedHeaders,AllowedOrigins, CorsOptions};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/tour")]
fn tour() -> Json<String> {
    Json(String::from("Tours Endpoint!"))
}

#[derive(Serialize, Deserialize)]
struct Booking {
    book: bool,
    cancel: bool,
    name: String,
    email: String,
    location: String,
}

#[post("/book", data = "<booking>")]
async fn book(booking: Json<Booking>) -> Json<String> {
    // todo something with booking
    match send_booking(&booking).await {
        Ok(_) => Json(String::from("Booking successful!")),
        Err(_) => Json(String::from("Booking failed!")),
    }
}

async fn send_booking(booking: &Booking) -> std::result::Result<(), amiquip::Error> {
    // Open connection
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5673")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Declare the exchange to publish to.
    let exchange = channel.exchange_declare(
        amiquip::ExchangeType::Topic,
        "bookings",
        ExchangeDeclareOptions::default(),
    )?;

    let routing_key = get_routing_key(booking);

    // Serialize Booking struct to JSON.
    let json_payload = match serde_json::to_string(booking) {
        Ok(payload) => payload,
        Err(err) => return Err(amiquip::Error::ServerClosedConnection { code: 0, message: String::from(format!("Error serializing booking: {}", err)) }),
    };

    let payload_bytes = json_payload.as_bytes();

    exchange.publish(Publish::new(payload_bytes, routing_key))?;

    connection.close()
}

fn get_routing_key(booking: &Booking) -> String {
    if (booking.book) {
        return String::from("tour.book");
    }
    return String::from("tour.cancel");
}

#[options("/book")]
fn book_options<'r>() -> Status {
    Status::Ok
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Access-Control-Allow-Origin",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "http://localhost:5173"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PUT, DELETE, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .attach(CORS)
        .mount("/", routes![index,tour,book,book_options])
        .launch()
        .await?;

    Ok(())
}