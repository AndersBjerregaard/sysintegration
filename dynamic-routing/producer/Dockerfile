FROM rustlang/rust:nightly-slim

ENV RABBITMQ_URI="amqp://guest:guest@localhost:5673/%2F"

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD ["producer"]
