#![feature(lazy_cell)] // This is a nightly-only experimental API. (lazy_cell #109736)

use lapin::{
    message::DeliveryResult,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
};
use rand::Rng;
use std::{sync::{Arc, LazyLock}, time::Duration};
use tokio::sync::{Mutex, MutexGuard};
use uuid::Uuid;

const DR_EXCHANGE_NAME: &str = "DR_Exchange";

// A value that is is initialized on the first access
static APP_UUID: LazyLock<Uuid> = LazyLock::new(|| Uuid::new_v4());

struct QueueInfo {
    default_prefix: String,
}

impl QueueInfo {
    pub fn new(default_prefix: &str) -> Self {
        QueueInfo {
            default_prefix: default_prefix.to_string(),
        }
    }

    pub fn get_app_uuid() -> String {
        String::from("consumer_queue_") + &APP_UUID.to_string()
    }

    pub fn get_tag_uuid() -> String {
        String::from("consumer_tag_") + &APP_UUID.to_string()
    }
}

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

    println!("--> Publishing initial ready message to load balancer...");

    publish_dr_message(&channel.lock().await).await;

    let consumer = consume_messages(&channel.lock().await).await;

    println!("--> Waiting for messages...");

    consumer.set_delegate(move |delivery: DeliveryResult| {
        let channel = channel.clone();

        async move {
            let delivery = match delivery {
                Ok(Some(delivery)) => delivery,
                Ok(None) => return,
                Err(error) => {
                    dbg!("--> Failed to consume queue message {}", error);
                    return;
                }
            };

            let byte_data = delivery.data.clone();
            let string_result = match std::str::from_utf8(&byte_data) {
                Ok(result) => result,
                Err(_) => "<Invalid UTF8 Bytes>",
            };

            let random = {
                let mut rng = rand::thread_rng();
                rng.gen_range(1..=5)
            };
            println!("--> Received message: {}", string_result);
            println!("--> Processing for {} seconds", random);
            std::thread::sleep(Duration::from_millis(random * 1000));

            println!("--> Process complete! Publishing message to load balancer...");

            publish_dr_message(&channel.lock().await).await;

            println!("--> Acknowledging...");

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("--> Failed to ack message");

            println!("--> Waiting for additional messages...");
        }
    });

    std::future::pending::<()>().await;
}

async fn publish_dr_message(channel: &MutexGuard<'_, Channel>) {
    let confirm = channel
        .basic_publish(
            DR_EXCHANGE_NAME,
            "",
            BasicPublishOptions::default(),
            &QueueInfo::get_app_uuid().as_bytes(),
            BasicProperties::default(),
        )
        .await;

    if confirm.is_err() {
        println!(
            "--> Could not publish message to load balancer: {}",
            confirm.unwrap_err()
        );
    } else {
        println!("--> Published message to load balancer!");
    }
}

async fn declare_dr_exchange(channel: &MutexGuard<'_, Channel>) {
    let mut exchange = channel
        .exchange_declare(
            DR_EXCHANGE_NAME,
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await;

    while exchange.is_err() {
        println!("--> Failed to declare exchange: {}", &exchange.unwrap_err());
        println!("--> Attempting to re-declare exchange in 3 seconds...");
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

async fn consume_messages(channel: &MutexGuard<'_, Channel>) -> Consumer {
    let mut queue = channel
        .queue_declare(
            &QueueInfo::get_app_uuid(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await;

    while queue.is_err() {
        println!("--> Failed to consume queue: {}", &queue.unwrap_err());
        println!("--> Attempting to re-declare queue in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        queue = channel
            .queue_declare(
                &QueueInfo::get_app_uuid(),
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    println!("--> Queue declared!");

    let mut consumer = channel
        .basic_consume(
            &QueueInfo::get_app_uuid(),
            &QueueInfo::get_tag_uuid(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await;

    while consumer.is_err() {
        println!("--> Failed to consume queue: {}", &consumer.unwrap_err());
        println!("--> Attempting to re-consume queue in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        consumer = channel
            .basic_consume(
                &QueueInfo::get_app_uuid(),
                &QueueInfo::get_tag_uuid(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    let consumer = consumer.unwrap();
    consumer
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