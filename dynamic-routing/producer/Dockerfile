FROM rustlang/rust:nightly-slim

ENV RABBITMQ_URI="amqp://guest:guest@localhost:5673/%2F"
ENV P_EXCH_NAME="not set"

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD ["producer"]
