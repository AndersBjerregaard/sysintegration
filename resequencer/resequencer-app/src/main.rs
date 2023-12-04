use std::sync::Arc;

use lapin::{
    message::DeliveryResult,
    options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, QueueDeclareOptions},
    types::{AMQPType, FieldTable},
    Connection, ConnectionProperties,
};
use tokio::sync::Mutex;

const QUEUE_NAME: &str = "q_resequencer";

#[tokio::main]
async fn main() {
    // An atomic reference counter to an asynchronous mutual exclusion wrapper.
    // For the sake of being able to have multiple threads safely access and mutate the wrapped data.
    let consumed_messages = Arc::new(Mutex::new(Vec::<String>::new()));

    let uri = "amqp://guest:guest@localhost:5673/%2F";

    let options = ConnectionProperties::default()
        .with_connection_name("resequencer_connection".to_string().into())
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = match Connection::connect(uri, options.clone()).await {
        Ok(con) => con,
        Err(e) => panic!(
            "[Critical] Could not establish connection to RabbitMQ: {}",
            e
        ),
    };

    let channel = match connection.create_channel().await {
        Ok(ch) => ch,
        Err(e) => panic!("[Critical] Could not create channel: {}", e),
    };

    let _queue = channel
        .queue_declare(
            QUEUE_NAME,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let consumer = channel
        .basic_consume(
            QUEUE_NAME,
            "aggregator_tag",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            // Create another pointer to the same allocation.
            let consumed_messages = Arc::clone(&consumed_messages);
            async move {
                let delivery = match delivery {
                    Ok(Some(delivery)) => delivery,
                    // The consumer got canceled
                    Ok(None) => return,
                    // Carries the error and is always followed by Ok(None)
                    Err(e) => {
                        println!("Failed to consume message: {}", e);
                        return;
                    }
                };

                let data_bytes = delivery.data.clone();
                let data_string = match std::str::from_utf8(&data_bytes) {
                    Ok(result) => result,
                    Err(_) => "<Invalid UTF8 Bytes>",
                };

                println!(
                    "[Information] Received message on '{}', with the body: '{}'",
                    QUEUE_NAME, data_string
                );
                println!("  Delivery tag: '{}'", delivery.delivery_tag);
                let headers = delivery.properties.headers().clone().unwrap();

                // Access the inner BTreeMap to perform lookups
                let amqp_value = headers.inner().get("messageId").unwrap();
                let supported_type = match amqp_value.get_type() {
                    AMQPType::LongString => true,
                    _ => {
                        println!("[Error] Unsupported header value type");
                        false
                    }
                };
                if !supported_type {
                    delivery
                        .nack(BasicNackOptions::default())
                        .await
                        .expect_err("[Error] Failed to reject message");
                    println!("[Information] Message rejected...");
                    return;
                }
                let binding = amqp_value.as_long_string().unwrap().clone();
                let header_value_bytes = binding.as_bytes();
                let header_value_string = std::str::from_utf8(header_value_bytes).unwrap();
                println!(
                    "  Header value of key 'messageId': '{}'",
                    header_value_string
                );
                // Locks this mutex, causing the current task to yield until the lock has been acquired.
                let mut messages = consumed_messages.lock().await;
                messages.push(data_string.to_string());
                // Hardcoded example of aggregating the values of two messages together, regardless of order
                if messages.len() == 2 {
                    let last = messages.pop().unwrap();
                    let first = messages.pop().unwrap();
                    println!(
                        "[Information] The last two messages' payloads aggregated: '{}{}'",
                        first, last
                    );
                }
                drop(messages); // Drop the reference, releasing this task's lock

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("[Error] Failed to acknowledge message");
                println!("[Information] Message acknowledged...");
            }
        }
    });

    // Block the main thread
    std::future::pending::<()>().await;
}
