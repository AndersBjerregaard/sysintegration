use core::time;
use std::{error::Error, thread};

use amiquip::{Connection, ExchangeDeclareOptions, Exchange, Channel, QueueDeclareOptions, ConsumerOptions, Publish, AmqpProperties};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5673")?;

    let channel = connection.open_channel(None)?;

    let exchange = channel.exchange_declare(
        amiquip::ExchangeType::Topic, 
        "rr-exchange", 
        ExchangeDeclareOptions::default()
    )?;

    let _consume_request = consume_request(&channel, &exchange).await?;

    Ok(())
}

async fn consume_request(channel: &Channel, exchange: &Exchange<'_>) -> Result<bool, Box<dyn Error>> {
    let queue = channel.queue_declare(
        "rr-request", 
        QueueDeclareOptions::default()
    )?;

    queue.bind(&exchange, "rr-request", amiquip::FieldTable::new())?;

    println!("--> Waiting for requests...\n");

    let consumer = queue.consume(ConsumerOptions::default())?;

    for (_i, message) in consumer.receiver().iter().enumerate() {
        match message {
            amiquip::ConsumerMessage::Delivery(delivery) => {
                let body = delivery.body.clone();
                let body = String::from_utf8_lossy(&body);
                println!("--> Request received: {}\n", body);
                consumer.ack(delivery)?;
                let body_text: String = body.to_string();
                println!("--> Processing request...\n");
                thread::sleep(time::Duration::from_millis(2000));
                let _reply = publish_reply(&exchange, &body_text).await?;
            },
            other => { println!("Consumer ended: {:?}\n", other); break; }
        }
    }

    Ok(true)
}

async fn publish_reply(exchange: &Exchange<'_>, request_text: &String) -> Result<bool, Box<dyn Error>> {
    let string_body = format!("Responding to: {}", request_text);
    let message = Publish::with_properties(
        string_body.as_bytes(), 
        "rr-reply", 
        AmqpProperties::default()
    );

    exchange.publish(message)?;

    println!("--> Sent response message\n");

    Ok(true)
}
