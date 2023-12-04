use std::{collections::HashMap, sync::Arc};

use lapin::{
    message::DeliveryResult,
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use tokio::sync::Mutex;

const QUEUE_NAME: &str = "q_resequencer";

#[derive(Debug)]
struct Sequence {
    payload: String,
    num_sequences: u64,
    sequence: u64,
    sequence_id: String,
}

impl Sequence {
    /// Creates a new [`Sequence`].
    fn new(payload: String, num_sequences: u64, sequence: u64, sequence_id: String) -> Self {
        Self {
            payload,
            num_sequences,
            sequence,
            sequence_id,
        }
    }
}

#[derive(Debug)]
struct Resequencer {
    sequences: HashMap<String, Vec<Option<String>>>,
}

impl Resequencer {
    fn new() -> Resequencer {
        Resequencer {
            sequences: HashMap::new(),
        }
    }

    fn push_sequence(&mut self, new_sequence: Sequence) {
        let sequence_id = &new_sequence.sequence_id;

        // If sequence_id exists, update or insert payload at the appropriate sequence number
        let sequence_list = self
            .sequences
            .entry(new_sequence.sequence_id.clone())
            .or_insert_with(|| vec![None; new_sequence.num_sequences as usize]);

        sequence_list[new_sequence.sequence as usize - 1] = Some(new_sequence.payload.clone());

        // Check if all sequences have been received for this sequence_id
        if sequence_list.iter().all(|s| s.is_some()) {
            let resequenced_payload: String = sequence_list
                .iter()
                .flatten()
                .map(|s| s.to_owned())
                .collect();

            println!("[Information] Received all sequences for sequence_id: {}", sequence_id);
            println!("[Information] Resequenced payload: {}", resequenced_payload);

            // Optionally, you can clear the sequences for this sequence_id after resequencing
            self.sequences.remove(sequence_id);
        }
    }
}

#[tokio::main]
async fn main() {
    // An atomic reference counter to an asynchronous mutual exclusion wrapper.
    // For the sake of being able to have multiple threads safely access and mutate the wrapped data.
    let consumed_messages = Arc::new(Mutex::new(Resequencer::new()));

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

                // Access the headers
                let headers = delivery.properties.headers().clone().unwrap();

                // Access the inner BTreeMap to perform lookups
                let b_tree_map = headers.inner();

                // Assume that the type is 'LongString' and reference the value thereafter.
                let binding = b_tree_map
                    .get("sequence_id")
                    .unwrap()
                    .as_long_string()
                    .unwrap();
                let sequence_id_bytes = binding.as_bytes();
                let sequence_id = std::str::from_utf8(sequence_id_bytes).unwrap();

                println!("  sequence_id: '{}'", &sequence_id);

                // Numbers from RabbitMQ message headers default to the AMQValue of 'LongLongInt', unless there's a decimal point, then it defaults to 'Double'
                let sequence = b_tree_map
                    .get("sequence")
                    .unwrap()
                    .as_long_long_int()
                    .unwrap();

                println!("  sequence: '{}'", &sequence);

                let num_sequences = b_tree_map
                    .get("num_sequences")
                    .unwrap()
                    .as_long_long_int()
                    .unwrap();

                println!("  num_sequences: '{}'", &num_sequences);

                let new_sequence = Sequence::new(
                    String::from(data_string),
                    num_sequences.try_into().unwrap(),
                    sequence.try_into().unwrap(),
                    String::from(sequence_id),
                );

                // Locks this mutex, causing the current task to yield until the lock has been acquired.
                let mut messages = consumed_messages.lock().await;

                messages.push_sequence(new_sequence);

                // Check if all sequences for a specific id has been consumed

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