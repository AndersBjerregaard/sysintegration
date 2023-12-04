use lapin::{
    message::DeliveryResult,
    options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, QueueDeclareOptions},
    types::{AMQPType, FieldTable},
    Connection, ConnectionProperties,
};

const QUEUE_NAME: &str = "q_aggregator";

#[tokio::main]
async fn main() {
    let uri = "amqp://guest:guest@localhost:5673/%2F";

    let options = ConnectionProperties::default()
        .with_connection_name("aggregator_connection".to_string().into())
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

                let byte_data = delivery.data.clone();
                let string_result = match std::str::from_utf8(&byte_data) {
                    Ok(result) => result,
                    Err(_) => "<Invalid UTF8 Bytes>",
                };

                println!(
                    "[Information] Received message on '{}', with the body: '{}'",
                    QUEUE_NAME, string_result
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
                let value = binding.as_bytes();
                let header_string_value = std::str::from_utf8(value).unwrap();
                println!("  Header value of key 'messageId': '{}'", header_string_value);

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
