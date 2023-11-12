use std::env;

use lapin::{ConnectionProperties, Connection};


#[tokio::main]
async fn main() {
    let uri: String = env::var("RABBITMQ_URI").unwrap();

    let options: ConnectionProperties = ConnectionProperties::default()
        .with_connection_name("producer_connection".to_string().into())
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection: Connection = connect_rabbitmq(&uri, options).await;
}

async fn connect_rabbitmq(uri: &str, options: ConnectionProperties) -> Connection {
    let mut connection: Result<Connection, lapin::Error> = Connection::connect(uri, options.clone()).await;


    while connection.is_err() {
        println!(
            "--> Failed to connect to rabbitmq: {}",
            &connection.unwrap_err()
        );
        connection = Connection::connect(uri, options.clone()).await;
    }

    let connection: Connection = connection.unwrap();
    connection
}