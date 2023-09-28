use std::{error::Error, io};

use amiquip::{Connection, Channel, ExchangeDeclareOptions, Exchange, Publish, AmqpProperties, QueueDeclareOptions, ConsumerOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5673")?;

    let channel = connection.open_channel(None)?;

    let exchange = channel.exchange_declare(
        amiquip::ExchangeType::Topic, 
        "rr-exchange", 
        ExchangeDeclareOptions::default()
    )?;

    let mut input = String::new();

    while input != "break" {
        println!("--> Write a message (write `break` to exit): ");
    
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
    
        let _request = send_request(&exchange, &input).await?;
    
        let _reply = await_reply(&channel, &exchange).await?;

        input = "".to_string();
    }

    Ok(())
}

async fn send_request(exchange: &Exchange<'_>, request_body: &String) -> Result<bool, Box<dyn Error>> {
    let string_body = String::from(request_body);
    let message = Publish::with_properties(
        string_body.as_bytes(), 
        "rr-request", 
        AmqpProperties::default().with_reply_to(String::from("rr-reply"))
    );

    exchange.publish(message)?;

    println!("--> Sent request message\n");

    Ok(true)
}

async fn await_reply(channel: &Channel, exchange: &Exchange<'_>) -> Result<bool, Box<dyn Error>> {
    let queue = channel.queue_declare(
        "rr-reply", 
        QueueDeclareOptions::default()
    )?;

    queue.bind(&exchange, "rr-reply", amiquip::FieldTable::new())?;

    let consumer = queue.consume(ConsumerOptions::default())?;

    println!("--> Waiting for reply...\n");

    for (_i, message) in consumer.receiver().iter().enumerate() {
        match message {
            amiquip::ConsumerMessage::Delivery(delivery) => {
                let body = delivery.body.clone();
                let body = String::from_utf8_lossy(&body);
                println!("--> Reply received: {}\n", body);
                consumer.ack(delivery)?;
                break;
            },
            other => { println!("Consumer ended: {:?}\n", other); break; }
        }
    }

    Ok(true)
}