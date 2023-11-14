use core::panic;
use std::env;

use lapin::{options::BasicPublishOptions, BasicProperties, Connection, ConnectionProperties};

#[tokio::main]
async fn main() {
    let uri: String = env::var("RABBITMQ_URI")
        .expect("[Critical] Environment variable `RABBITMQ_URI` unassigned!");
    println!("[Info] RabbitMQ uri was loaded as `{}`", &uri);

    let exchange_name: String =
        env::var("P_EXCH_NAME").expect("[Critical] Environment variable `P_EXCH_NAME` unassigned!");
    if exchange_name == "not set" {
        panic!("[Critical] Environment variable `P_EXCH_NAME` was not set! Aborting...");
    } else {
        print!(
            "[Info] Environment variable `P_EXCH_NAME` was loaded as `{}`",
            exchange_name
        );
    }

    let options: ConnectionProperties = ConnectionProperties::default()
        .with_connection_name("producer_connection".to_string().into())
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection: Connection = connect_rabbitmq(&uri, options).await;

    let channel = match connection.create_channel().await {
        Ok(ch) => ch,
        Err(e) => panic!("[Critical] Could not create channel: {}", e),
    };

    let payload = "test payload".as_bytes();

    // Empty string first argument to route to default exchange.
    // Routing key is set to the target queue, when targeting default exchange.
    let confirm = channel.basic_publish(
        "",
        &exchange_name,
        BasicPublishOptions::default(),
        payload,
        BasicProperties::default(),
    ).await;

    if confirm.is_err() {
        println!("[Error] Could not publish message: {}", confirm.as_ref().unwrap_err());
    } else {
        println!("[Info] Published message!");
    }
}

async fn connect_rabbitmq(uri: &str, options: ConnectionProperties) -> Connection {
    let mut connection: Result<Connection, lapin::Error> =
        Connection::connect(uri, options.clone()).await;
    let mut interval = 3;
    let mut retries = 0;

    while connection.is_err() {
        if retries == 10 {
            core::panic!("[Critical] Failed to connect to RabbitMQ after 10 retries! Aborting...");
        }
        println!(
            "[Error] Failed to connect to RabbitMQ: {}",
            &connection.unwrap_err()
        );
        println!("[Info] Attempting to reconnect after {} seconds", interval);
        interval += 2;
        retries += 1;

        connection = Connection::connect(uri, options.clone()).await;
    }

    connection.unwrap()
}
