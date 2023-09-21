use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result, ExchangeDeclareOptions};

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
            ..QueueDeclareOptions::default()
        },
    )?;
    println!("created exclusive queue {}", queue.name());

    queue.bind(
        &exchange, 
        "tour.book", 
        amiquip::FieldTable::new()
    )?;

    // Start a consumer. Use no_ack: true so the server doesn't wait
    // for this app to ack the message it sends.
    let consumer = queue.consume(ConsumerOptions { 
        no_ack: true,
        ..ConsumerOptions::default()
    })?;

    println!("Waiting for bookings. Press Ctrl+C to exit.");

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                println!("({:>3}) {}:{}", i, delivery.routing_key, body);
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()
}