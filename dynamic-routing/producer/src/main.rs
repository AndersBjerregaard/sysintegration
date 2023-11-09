use std::{env, time::Duration};

use lapin::{
    Channel,
    Connection, ConnectionProperties, Error,
};

#[tokio::main]
async fn main() {
    let uri: String = match env::var("RABBITMQ_URI") {
        Ok(t) => t,
        Err(e) => {
            println!(
                "Failed to match RABBITMQ_URI environment variable: {}. Defaulting to `amqp://guest:guest@localhost:5673/#2F`",
                e
            );
            "amqp://guest:guest@localhost:5673/#2F".to_string()
        }
    };

    let options: ConnectionProperties = ConnectionProperties::default()
        .with_connection_name("producer-connection".to_string().into())
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection: Connection = connect_rabbitmq(&uri, options).await;

    let channel: Channel = connection.create_channel().await.unwrap();

    println!("Created channel!");

    // let mut input: String = String::new();
    // loop {
    //     println!("--> Enter a message to be produced> ");
    //     match io::stdin().read_line(&mut input) {
    //         Ok(_) => {}
    //         Err(e) => {
    //             println!("--> Failed to read entered message: {}", e);
    //             continue;
    //         }
    //     };

    //     let payload = input.as_bytes();

    //     let confirm: Result<PublisherConfirm, Error> = channel
    //         .basic_publish(
    //             "EXCHANGE_NAME",
    //             "",
    //             BasicPublishOptions::default(),
    //             payload,
    //             BasicProperties::default(),
    //         )
    //         .await;

    //     if confirm.is_err() {
    //         println!("--> Could not publish message: {}", &confirm.unwrap_err());
    //     } else {
    //         println!("--> Published message: `{}`", &input);
    //     }
    // }
}

async fn connect_rabbitmq(uri: &str, options: ConnectionProperties) -> Connection {
    let mut connection: Result<Connection, Error> = Connection::connect(uri, options.clone()).await;
    let mut retries = 0;
    let mut interval = 5;
    while connection.is_err() && retries < 10 {
        println!(
            "--> Failed to connect to rabbitmq: {}",
            &connection.unwrap_err()
        );
        println!(
            "--> Retry attempt number {}: Attempting to reconnect in {} seconds",
            retries, interval
        );
        std::thread::sleep(Duration::from_millis(interval));
        connection = Connection::connect(uri, options.clone()).await;
        retries += 1;
        interval += 5;
    }
    if retries == 10 {
        panic!("Could not connect to rabbitmq after 10 retries!");
    }
    let connection: Connection = connection.unwrap();
    return connection;
}
