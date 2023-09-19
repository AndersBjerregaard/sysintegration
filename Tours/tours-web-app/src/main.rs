#[macro_use] extern crate rocket;

use amiquip::{Connection, Exchange, Publish, Result};
use rocket::form::Form;
use rocket::response::content;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/tour")]
fn tour() -> content::RawJson<String> {
    content::RawJson(String::from("Tours Endpoint!"))
}

#[derive(FromForm)]
struct Task<'r> {
    complete: bool,
    r#type: &'r str,
}

#[post("/book", data = "<task>")]
async fn book(task: Form<Task<'_>>) -> content::RawJson<String> {
    // todo something with task
    match send_booking().await {
        Ok(_) => content::RawJson("Booking successful!".to_string()),
        Err(_) => content::RawJson("Booking failed!".to_string()),
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

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/", routes![book,tour,index])
        .launch()
        .await
        .expect("Rocket did not launch successfully");
}