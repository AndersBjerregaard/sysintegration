use std::default;

use amiquip::{Connection, QueueDeclareOptions, Result, Exchange, FieldTable, ConsumerOptions, ConsumerMessage};

fn main() -> Result<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5673")?;
    
    let channel = connection.open_channel(None)?;

    // Declare DLX exchange
    let dead_letter_exchange = channel.exchange_declare(
        amiquip::ExchangeType::Fanout, 
        "dead-letter-exchange", 
        amiquip::ExchangeDeclareOptions { 
            durable: true, 
            ..amiquip::ExchangeDeclareOptions::default() 
        }
    )?;

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
        FieldTable::new()
    )?;

    let consumer = dead_letter_queue.consume(ConsumerOptions::default())?;

    println!("Waiting for error messages from the dead letter queue. Press Ctrl+C to exit.");

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);

                println!("Message received on dead-letter queue: {}", body);

                // Acknowledge the message to remove it from the dead-letter queue.
                delivery.ack(&channel)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()?;

    Ok(())
}