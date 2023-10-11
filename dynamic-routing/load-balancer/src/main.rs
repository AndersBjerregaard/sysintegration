use std::{sync::Arc, time::Duration};

use lapin::{
    options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties,
};
use tokio::sync::{Mutex, MutexGuard};

const DR_EXCHANGE_NAME: &str = "DR_Exchange";
const DR_QUEUE_NAME: &str = "DR_Queue";

#[tokio::main]
async fn main() {
    let uri = "amqp://guest:guest@localhost:5673/%2F";

    let options = ConnectionProperties::default()
        .with_connection_name("producer-connection".to_string().into())
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = connect_rabbitmq(uri, options).await;

    let channel = Arc::new(Mutex::new(connection.create_channel().await.unwrap()));

    declare_dr_exchange(&channel.lock().await).await;

    declare_and_bind_dr_queue(&channel.lock().await).await;

    // Block the main thread
    std::future::pending::<()>().await;
}

async fn declare_and_bind_dr_queue(lock: &MutexGuard<'_, Channel>) {
    let mut queue = lock
        .queue_declare(
            DR_QUEUE_NAME,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await;

    while queue.is_err() {
        println!("--> Failed to declare dr_queue: {}", &queue.unwrap_err());
        println!("--> Attempting to re-declare dr_queue in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        queue = lock
            .queue_declare(
                DR_QUEUE_NAME,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    println!("--> dr_queue declared!");

    let mut queue_bind_result = lock.queue_bind(
        DR_QUEUE_NAME,
        DR_EXCHANGE_NAME,
        "",
        QueueBindOptions::default(),
        FieldTable::default(),
    )
    .await;

    while queue_bind_result.is_err() {
        println!("--> Failed to bind dr_queue to dr_exchange: {}", queue_bind_result.unwrap_err());
        println!("--> Attempting to re-bind in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        queue_bind_result = lock.queue_bind(
            DR_QUEUE_NAME,
            DR_EXCHANGE_NAME,
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await;
    }

    println!("--> dr_queue bound to dr_exchange!");
}

async fn declare_dr_exchange(channel: &MutexGuard<'_, Channel>) {
    let mut exchange = channel
        .exchange_declare(
            DR_EXCHANGE_NAME,
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await;

    while exchange.is_err() {
        println!(
            "--> Failed to declare dr_exchange: {}",
            &exchange.unwrap_err()
        );
        println!("--> Attempting to re-declare dr_exchange in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        exchange = channel
            .exchange_declare(
                DR_EXCHANGE_NAME,
                lapin::ExchangeKind::Direct,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    println!("--> DR_Exchange declared!");
}

async fn connect_rabbitmq(uri: &str, con_props: ConnectionProperties) -> Connection {
    let mut connection = Connection::connect(uri, con_props.clone()).await;

    while connection.is_err() {
        println!(
            "--> Failed to connect to rabbitmq: {}",
            &connection.unwrap_err()
        );
        println!("--> Attempting to reconnect in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        connection = Connection::connect(uri, con_props.clone()).await;
    }
    println!("--> Connected to rabbitmq!");
    // There might be a way to register a callback to handle dropped connections here
    let connection = connection.unwrap();
    connection
}
