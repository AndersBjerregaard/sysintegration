use std::{io, sync::Arc, thread, time::Duration};

use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions}, BasicProperties, Channel, Connection, ConnectionProperties, types::FieldTable,
};
use tokio::sync::{Mutex, MutexGuard};

const PRODUCER_EXCHANGE_NAME: &str = "Producer_Exchange";

#[tokio::main]
async fn main() {
    let uri = "amqp://guest:guest@localhost:5673/%2F";

    let options = ConnectionProperties::default()
        .with_connection_name("producer-connection".to_string().into())
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = connect_rabbitmq(uri, options).await;

    let channel = Arc::new(Mutex::new(connection.create_channel().await.unwrap()));

    declare_producer_exchange(&channel.lock().await).await;

    loop {
        println!("--> Enter a message:\n> ");
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(error) => {
                println!("--> Failed to read the entered message: {}", error);
                continue;
            }
        };

        let payload = input.as_bytes();

        let confirm = &channel
            .lock()
            .await
            .basic_publish(
                PRODUCER_EXCHANGE_NAME,
                "",
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await;

        if confirm.is_err() {
            println!("--> Could not publish message: {}", confirm.as_ref().unwrap_err());
        } else {
            println!("--> Published message!");
        }
    }
}

async fn declare_producer_exchange(lock: &MutexGuard<'_, Channel>) {
    let mut exchange = lock
        .exchange_declare(
            PRODUCER_EXCHANGE_NAME,
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await;

    while exchange.is_err() {
        println!(
            "--> Failed to declare producer_exchange: {}",
            &exchange.unwrap_err()
        );
        println!("--> Attempting to re-declare producer_exchange in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        exchange = lock
            .exchange_declare(
                PRODUCER_EXCHANGE_NAME,
                lapin::ExchangeKind::Direct,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    println!("--> producer_exchange declared!");
}

async fn connect_rabbitmq(uri: &str, con_props: ConnectionProperties) -> Connection {
    let mut connection = Connection::connect(uri, con_props.clone()).await;

    while connection.is_err() {
        println!(
            "--> Failed to connect to rabbitmq: {}",
            &connection.unwrap_err()
        );
        println!("--> Attempting to reconnect in 3 seconds...");
        thread::sleep(Duration::from_millis(3000));
        connection = Connection::connect(uri, con_props.clone()).await;
    }
    println!("--> Connected to rabbitmq!");
    // There might be a way to register a callback to handle dropped connections here
    let connection = connection.unwrap();
    connection
}
