use core::panic;
use std::env;

use lapin::{ConnectionProperties, Connection};


#[tokio::main]
async fn main() {
    let uri: String = env::var("RABBITMQ_URI").unwrap();
    println!("[Info] RabbitMQ uri was loaded as `{}`", &uri);

    let exchange_name: String = env::var("P_EXCH_NAME").unwrap();
    if exchange_name == "not set" {
        panic!("[Critical] Environment variable `P_EXCH_NAME` was not set! Aborting...");
    } else {
        print!("[Info] Environment variable `P_EXCH_NAME` was loaded as `{}`", exchange_name);
    }

    let options: ConnectionProperties = ConnectionProperties::default()
        .with_connection_name("producer_connection".to_string().into())
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection: Connection = connect_rabbitmq(&uri, options).await;
}

async fn connect_rabbitmq(uri: &str, options: ConnectionProperties) -> Connection {
    let mut connection: Result<Connection, lapin::Error> = Connection::connect(uri, options.clone()).await;
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