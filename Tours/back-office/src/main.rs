use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result, ExchangeDeclareOptions, Exchange};
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

    // Declare DLX exchange
    let dead_letter_exchange = channel.exchange_declare(
        amiquip::ExchangeType::Fanout, 
        "dead-letter-exchange", 
        ExchangeDeclareOptions { 
            durable: true, 
            ..ExchangeDeclareOptions::default() 
        }
    )?;

    // Instantiate optional arguments as a FieldTable for the queue
    // Map the name of the exchange to the key of `x-dead-letter-exchange` to set DLX for this specific queue.
    let mut arguments = amiquip::FieldTable::new();
    arguments.insert(
        "x-dead-letter-exchange".to_string(),
        amiquip::AmqpValue::LongString("bookings".to_string()),
    );

    // Declare DLX queue
    let dead_letter_queue = channel.queue_declare(
        "dead-letter-queue", 
        QueueDeclareOptions { 
            durable: true, 
            arguments, 
            ..QueueDeclareOptions::default() 
        }
    )?;

    dead_letter_queue.bind(
        &dead_letter_exchange, 
        "booking.error", 
        amiquip::FieldTable::new()
    )?;

    // Declare the exchange.
    let exchange: Exchange<'_> = channel.exchange_declare(
        amiquip::ExchangeType::Topic,
        "bookings",
        ExchangeDeclareOptions { 
            durable: true, 
            ..ExchangeDeclareOptions::default() 
        },
    )?;


    // Declare the exclusive, server-named queue to use to consume.
    let queue = channel.queue_declare(
        "bookings-queue",
        QueueDeclareOptions {
            exclusive: true,
            durable: true,
            ..QueueDeclareOptions::default()
        },
    )?;

    // Bind to multiple routing keys
    let routing_keys = vec!["tour.book", "tour.cancel"];

    // You may also specify a routing key to use when the messages are being dead-lettered. 
    // If the routing key is not set, the message's own routing keys are used. Source: https://www.rabbitmq.com/dlx.html#routing
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
                        eprintln!("Unable to deserialize message body data. Rejecting without requeuing...");
                        delivery.nack(&channel, false)?;
                        println!("Publishing error to the dead letter queue...");
                        let error_information: String = format!("{} Back-Office Application error serializing message information", chrono::prelude::Local::now());
                        let error_information: &[u8] = error_information.as_bytes();
                        let dead_letter_message = amiquip::Publish::with_properties(
                            error_information, 
                            "booking.error", 
                            amiquip::AmqpProperties::default().with_delivery_mode(2)
                        );
                        dead_letter_exchange.publish(dead_letter_message)?;
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