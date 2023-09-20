#[macro_use] extern crate rocket;

use amiquip::{Connection, Exchange, Publish, Result};
use rocket::form::Form;
use rocket::response::content;
use rocket::serde::Deserialize;
use rocket::serde::json::Json;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/tour")]
fn tour() -> Json<String> {
    Json(String::from("Tours Endpoint!"))
}

// Struct for form data
// #[derive(FromForm)]
// struct Booking {
//     book: bool,
//     cancel: bool,
//     name: String,
//     email: String,
//     location: String,
// }

#[derive(Deserialize)]
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
    match send_booking().await {
        Ok(_) => Json(String::from("Booking successful!")),
        Err(_) => Json(String::from("Booking failed!")),
    }
}

async fn send_booking() -> std::result::Result<(), amiquip::Error> {
    // Open connection
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

    // Publish a message to the "hello" queue.
    exchange.publish(Publish::new("hello there".as_bytes(), "hello"))?;

    connection.close()
}

// #[launch]
// fn rocket() -> _ {
//     rocket::build()
//         .mount("/", routes![index,tour,book])
// }

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index,tour,book])
        .launch()
        .await?;

    Ok(())
}