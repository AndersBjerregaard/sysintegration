# Topic Exchange Message Broker Development Showcase

The following commands expects the current working directory to be in the `Tours` directory.

First off you need a RabbitMQ message broker instance running.
The port exposure for the host system is offset from the default ports being used by message broker protocols, in case the host system already are using these.

```
docker run -d --hostname my-rabbit --name ecomm-rabbit -p 15672:15672 -p 5672:5672 rabbitmq:3-management
```

Start Vue frontend application

```
cd frontend-app/
npm install
npm run dev
```

Now, start the message consumers.
The email-service only receives messages that have to do with bookings (not cancellations).
Open another shell session in the `Tours` directory

```
cd email-service/
cargo build
cargo run
```

Open another shell session in the `Tours` directory
The back-office receives all types of bookings (cancellations as well).

```
cd back-office/
cargo build
cargo run
```

Finally, start the rocket web api, that also acts as the topic exchange publisher.

```
cd tours-web-app/
cargo build
cargo run
```

Now, you can open a browser and open the url `http://localhost:5173/`, fill out the form and see the consumer services react to the form submissions.
Additionally, you can open the url `http://localhost:15673/` to see the rabbitmq management interface to get additional information about the bindings, messages and exchange.
