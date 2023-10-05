use std::{thread, time::Duration};

use lapin::{ConnectionProperties, Connection, options::{ExchangeDeclareOptions, QueueDeclareOptions, BasicPublishOptions}, types::FieldTable, BasicProperties};

#[tokio::main]
async fn main() {
    let uri = "amqp://guest:guest@localhost:5673/%2F";
    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio)
        .with_connection_name("nam-nam".into());

    let connection = Connection::connect(uri, options).await.unwrap();

    println!("CONNECTED");

    let channel = connection.create_channel().await.unwrap();

    let _exchange = channel.exchange_declare("e_pdf", lapin::ExchangeKind::Fanout, ExchangeDeclareOptions::default(), FieldTable::default()).await;

    let _queue = channel.queue_declare("q_pdf", QueueDeclareOptions::default(), FieldTable::default()).await.unwrap();

    let payload = b"Hello pdf!";

    loop {
        println!("Publishing message...");
        let _confirm = channel.basic_publish("e_pdf", "q_pdf", BasicPublishOptions::default(), payload, BasicProperties::default()).await.unwrap();
        thread::sleep(Duration::from_millis(2000));
    }

    // std::future::pending::<()>().await;
}