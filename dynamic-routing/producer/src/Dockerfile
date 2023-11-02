FROM rustlang/rust:nightly-alpine

# Environment variable for setting rabbitmq connection uri

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD [ "producer" ]