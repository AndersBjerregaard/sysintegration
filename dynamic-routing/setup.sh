#!/bin/bash

docker compose down

docker compose up -d

echo "Waiting for RabbitMQ management API to be ready..."

sleep 3

docker exec rabbitmq rabbitmqctl status

# Source configuration file to load variables
source rabbit-infrastructure.cfg

rabbitmqadmin declare queue name="$PROD_TO_BALANCER_QUEUE_NAME" durable=false