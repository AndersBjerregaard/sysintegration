#[macro_use] extern crate rocket;

use amiquip::{Connection, Exchange, Publish, Result, ExchangeDeclareOptions, AmqpProperties};
use rocket::fairing::{Fairing, Kind, Info};
use rocket::http::{Header, Status};
use rocket::{Request, Response};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;

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

// Structural composition inheritance of booking to allow classes (business, economic, first, etc.)
#[derive(Serialize, Deserialize)]
struct BookingV2 {
    booking: Booking,
    class: String,
}

#[post("/book", data = "<booking>")]
async fn book(booking: Json<Booking>) -> Json<String> {
    let routing_key = get_routing_key(&booking);

    let json_payload: String = match get_serialized_booking(&booking) {
        Ok(payload) => payload,
        Err(_) => return Json(String::from("Error serializing booking")),
    };

    let bytes_payload = json_payload.as_bytes();

    match send_booking(routing_key, bytes_payload).await {
        Ok(_) => Json(String::from("Booking or cancellation successful!")),
        Err(_) => Json(String::from("Unexpected error occurred")),
    }
}

// This endpoint has the purpose of showcasing a message that the consumers are unable to process.
// Since they don't have logic to process a BookingV2 struct.
#[post("/bookv2", data = "<bookingv2>")]
async fn bookv2(bookingv2: Json<BookingV2>) -> Json<String> {
    let routing_key = get_routing_key_v2(&bookingv2);

    let json_payload: String = match get_serialized_booking_v2(&bookingv2) {
        Ok(payload) => payload,
        Err(_) => return Json(String::from("Error serializing booking")),
    };

    let bytes_payload = json_payload.as_bytes();

    match send_booking(routing_key, bytes_payload).await {
        Ok(_) => Json(String::from("Booking (Version 2) or cancellation successful!")),
        Err(_) => Json(String::from("Unexpected error occurred")),
    }
}

async fn send_booking(routing_key: String, bytes_payload: &[u8]) -> std::result::Result<(), amiquip::Error> {
    // Open connection
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5673")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Enable synchronous publisher confirms on the channel.
    channel.enable_publisher_confirms()?;

    // Declare the exchange to publish to.
    let exchange: Exchange<'_> = channel.exchange_declare(
        amiquip::ExchangeType::Topic,
        "bookings",
        ExchangeDeclareOptions::default(),
    )?;

    // Create persistent message. Delivery mode can be set to `1` for transient and `2` for persistent.
    let message = Publish::with_properties(bytes_payload, routing_key, AmqpProperties::default().with_delivery_mode(2));

    // Publish persistent message
    exchange.publish(message)?;

    // Wait for confirm from the RabbitMQ server.
    if channel.listen_for_publisher_confirms().is_ok() {
        println!("Message sent and confirmed by RabbitMQ server.");
    } else {
        eprintln!("Message sending failed or was not confirmed by RabbitMQ server.");
    }

    connection.close()
}

fn get_routing_key(booking: &Booking) -> String {
    if booking.book {
        return String::from("tour.book");
    }
    return String::from("tour.cancel");
}

fn get_routing_key_v2(booking: &BookingV2) -> String {
    if booking.booking.book {
        return String::from("tour.book");
    }
    return String::from("tour.cancel");
}

fn get_serialized_booking(booking: &Booking) -> Result<std::string::String, serde_json::Error> {
    match serde_json::to_string(booking) {
        Ok(it) => Ok(it),
        Err(err) => return Err(err),
    }
}

fn get_serialized_booking_v2(booking: &BookingV2) -> Result<std::string::String, serde_json::Error> {
    match serde_json::to_string(booking) {
        Ok(it) => Ok(it),
        Err(err) => return Err(err),
    }
}

#[options("/book")]
fn book_options<'r>() -> Status {
    Status::Ok
}

#[options("/bookv2")]
fn bookv2_options<'r>() -> Status {
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
        .mount("/", routes![index,tour,book,book_options,bookv2,bookv2_options])
        .launch()
        .await?;

    Ok(())
}