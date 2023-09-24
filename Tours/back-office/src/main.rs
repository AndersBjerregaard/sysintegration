use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result, ExchangeDeclareOptions};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Booking {
    book: bool,
    cancel: bool,
    name: String,
    email: String,
    location: String,
}

impl Booking {
    fn new(book: bool, 
        cancel: bool, 
        name: String, 
        email: String, 
        location: String) -> Booking {
            Booking {
                book: book,
                cancel: cancel,
                name: name,
                email: email,
                location: location,
            }
    }
}

fn main() -> Result<()> {
    // Open connection.
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5673")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Declare the exchange.
    let exchange = channel.exchange_declare(
        amiquip::ExchangeType::Topic, 
        "bookings", 
        ExchangeDeclareOptions::default()
    )?;

    // Declare the exclusive, server-named queue to use to consume.
    let queue = channel.queue_declare(
        "",
        QueueDeclareOptions {
            exclusive: true,
            durable: true,
            ..QueueDeclareOptions::default()
        },
    )?;
    println!("created exclusive queue {}", queue.name());

    // Bind to multiple routing keys
    let routing_keys = vec!["tour.book", "tour.cancel"];

    for routing_key in routing_keys {
        queue.bind(
            &exchange, 
            routing_key, 
            amiquip::FieldTable::new()
        )?;
    }

    // Disable no_ack to allow manual acknowledgement
    let consumer = queue.consume(ConsumerOptions { 
        no_ack: false,
        ..ConsumerOptions::default()
    })?;

    println!("Waiting for bookings. Press Ctrl+C to exit.");

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);

                println!("Message received with routing key and data: {}:{}", delivery.routing_key, body);
                
                match process(body) {
                    Ok(_) => { 
                        println!("Successfully deserialized message body data. Acknowledging..."); 
                        delivery.ack(&channel)?
                    },
                    Err(_) => {
                        println!("Unable to deserialize message  body data. Rejecting without requeuing...");
                        delivery.nack(&channel, false)?
                    },
                };
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()
}

fn process(unprocessed_string: std::borrow::Cow<'_, str>) -> std::result::Result<Booking, serde_json::Error> {
    // Dereference string from body
    let body = unprocessed_string.into_owned();

    serde_json::from_str(&body)
}