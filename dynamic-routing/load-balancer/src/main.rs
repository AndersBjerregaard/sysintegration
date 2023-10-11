use std::{sync::Arc, time::Duration};

use lapin::{
    message::DeliveryResult,
    options::{
        BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, Connection, ConnectionProperties,
};
use tokio::sync::{Mutex, MutexGuard};

const DR_EXCHANGE_NAME: &str = "DR_Exchange";
const PRODUCER_EXCHANGE_NAME: &str = "Producer_Exchange";
const DR_QUEUE_NAME: &str = "DR_Queue";
const PRODUCER_QUEUE_NAME: &str = "Producer_Queue";

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

    declare_and_bind_producer_queue(&channel.lock().await).await;

    let producer_consumer = consume_producer_messages(&channel.lock().await).await;

    declare_dr_exchange(&channel.lock().await).await;

    declare_and_bind_dr_queue(&channel.lock().await).await;

    let dr_consumer = consume_dr_messages(&channel.lock().await).await;

    println!("--> Ready to receive messages on producer_queue...");

    producer_consumer.set_delegate(move |delivery_result: DeliveryResult| async move {
        let delivery = match delivery_result {
            Ok(Some(delivery)) => delivery,
            Ok(None) => return,
            Err(error) => {
                println!("--> Failed to consume message {}", error);
                return;
            }
        };

        let byte_data = delivery.data.clone();
        let string_result = match std::str::from_utf8(&byte_data) {
            Ok(result) => result,
            Err(_) => "<Invalid UTF8 Bytes>",
        };

        println!("--> Received message on producer_queue: {}", string_result);

        println!("--> Acknowledging...");

        delivery
            .ack(BasicAckOptions::default())
            .await
            .expect("--> Failed to ack message");

        println!("--> Ready to receive additional message on producer_queue...");
    });

    println!("--> Ready to receive messages on dr_queue...");

    dr_consumer.set_delegate(move |delivery_result: DeliveryResult| {
        let dr_channel = channel.clone();

        async move {
            let delivery = match delivery_result {
                Ok(Some(delivery)) => delivery,
                Ok(None) => return,
                Err(error) => {
                    println!("--> Failed to consume message {}", error);
                    return;
                }
            };

            let byte_data = delivery.data.clone();
            let string_result = match std::str::from_utf8(&byte_data) {
                Ok(result) => result,
                Err(_) => "<Invalid UTF8 Bytes>",
            };

            println!("--> Received message on dr_queue: {}", string_result);

            println!("--> Acknowledging...");

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("--> Failed to ack message");

            println!("--> Ready to receive additional message on dr_queue...");
        }
    });

    // Block the main thread
    std::future::pending::<()>().await;
}

async fn consume_producer_messages(lock: &MutexGuard<'_, Channel>) -> lapin::Consumer {
    let mut consumer = lock
        .basic_consume(
            PRODUCER_QUEUE_NAME,
            "load_balancer_1",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await;

    while consumer.is_err() {
        println!(
            "--> Failed to consume producer_queue: {}",
            &consumer.unwrap_err()
        );
        println!("--> Attempting to re-consume producer_queue in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        consumer = lock
            .basic_consume(
                PRODUCER_QUEUE_NAME,
                "load_balancer_1",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    println!("--> Established consumation of producer_queue!");

    consumer.unwrap()
}

async fn declare_and_bind_producer_queue(lock: &MutexGuard<'_, Channel>) {
    let mut queue = lock
        .queue_declare(
            PRODUCER_QUEUE_NAME,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await;

    while queue.is_err() {
        println!(
            "--> Failed to declare producer_queue: {}",
            &queue.unwrap_err()
        );
        println!("--> Attempting to re-declare prodcuer_queue in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        queue = lock
            .queue_declare(
                PRODUCER_QUEUE_NAME,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    println!("--> producer_queue declared!");

    let mut queue_bind_result = lock
        .queue_bind(
            PRODUCER_QUEUE_NAME,
            PRODUCER_EXCHANGE_NAME,
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await;

    while queue_bind_result.is_err() {
        println!(
            "--> Failed to bind producer_queue to producer_exchange: {}",
            queue_bind_result.unwrap_err()
        );
        println!("--> Attempting to re-bind in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        queue_bind_result = lock
            .queue_bind(
                PRODUCER_QUEUE_NAME,
                PRODUCER_EXCHANGE_NAME,
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    println!("--> producer_queue bound to dr_exchange!");
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

async fn consume_dr_messages(lock: &MutexGuard<'_, Channel>) -> lapin::Consumer {
    let mut consumer = lock
        .basic_consume(
            DR_QUEUE_NAME,
            "load_balancer_2",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await;

    while consumer.is_err() {
        println!("--> Failed to consume dr_queue: {}", &consumer.unwrap_err());
        println!("--> Attempting to re-consume dr_queue in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        consumer = lock
            .basic_consume(
                DR_QUEUE_NAME,
                "load_balancer_2",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await;
    }

    println!("--> Established consumation of dr_queue!");

    consumer.unwrap()
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

    let mut queue_bind_result = lock
        .queue_bind(
            DR_QUEUE_NAME,
            DR_EXCHANGE_NAME,
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await;

    while queue_bind_result.is_err() {
        println!(
            "--> Failed to bind dr_queue to dr_exchange: {}",
            queue_bind_result.unwrap_err()
        );
        println!("--> Attempting to re-bind in 3 seconds...");
        std::thread::sleep(Duration::from_millis(3000));
        queue_bind_result = lock
            .queue_bind(
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

async fn declare_dr_exchange(lock: &MutexGuard<'_, Channel>) {
    let mut exchange = lock
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
        exchange = lock
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
