use std::{time::Duration, sync::Arc};
use lapin::{Connection, ConnectionProperties, Consumer, Channel, options::{BasicConsumeOptions, BasicAckOptions, QueueDeclareOptions, ExchangeDeclareOptions, BasicPublishOptions}, types::FieldTable, message::DeliveryResult, BasicProperties};
use tokio::sync::{Mutex, MutexGuard};
use uuid::Uuid;

const EXCHANGE_NAME: &str = "DR_Exchange";

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

            println!("--> Message received! Processing for 4 seconds...");
            std::thread::sleep(Duration::from_millis(4000));

            println!("--> Process complete! Publishing message to load balancer...");

            // Acquire lock before using the channel
            let channel = channel.lock().await;

            let confirm = channel.basic_publish(
                EXCHANGE_NAME,
                "",
                BasicPublishOptions::default(),
                b"Ready!",
                BasicProperties::default(),
            )
            .await;

            if confirm.is_err() {
                println!("--> Could not publish message to load balancer: {}", confirm.unwrap_err());
            } else {
                println!("--> Published message to load balancer!");
            }

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

async fn declare_dr_exchange(channel: &MutexGuard<'_, Channel>) {
    let mut exchange = channel.exchange_declare(
        EXCHANGE_NAME, 
        lapin::ExchangeKind::Direct, 
        ExchangeDeclareOptions::default(), 
        FieldTable::default()).await;

    while exchange.is_err() {
        println!("--> Failed to declare exchange: {}", &exchange.unwrap_err());
        println!("--> Attempting to re-declare exchange in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        exchange = channel.exchange_declare(
            EXCHANGE_NAME, 
            lapin::ExchangeKind::Direct, 
            ExchangeDeclareOptions::default(), 
            FieldTable::default()).await;
    }

    println!("--> Exchange declared!");
}

async fn consume_messages(channel: &MutexGuard<'_, Channel>) -> Consumer {
    let consumer_tag = Uuid::new_v4().to_string();
    let queue_name = Uuid::new_v4().to_string();

    let mut queue = channel.queue_declare(
        &queue_name, 
        QueueDeclareOptions::default(), 
        FieldTable::default()).await;

    while queue.is_err() {
        println!("--> Failed to consume queue: {}", &queue.unwrap_err());
        println!("--> Attempting to re-declare queue in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        queue = channel.queue_declare(
            &queue_name, 
            QueueDeclareOptions::default(), 
            FieldTable::default()).await;
    }

    println!("--> Queue declared!");

    let mut consumer = channel
        .basic_consume(
            &queue_name, 
            &consumer_tag, 
            BasicConsumeOptions::default(), 
            FieldTable::default()).await;

    while consumer.is_err() {
        println!("--> Failed to consume queue: {}", &consumer.unwrap_err());
        println!("--> Attempting to re-consume queue in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        consumer = channel.basic_consume(&queue_name, &consumer_tag, BasicConsumeOptions::default(), FieldTable::default()).await;
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