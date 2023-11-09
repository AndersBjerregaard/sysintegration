#!/bin/bash

docker compose down

docker compose --env-file ./.env up -d

echo "Waiting for RabbitMQ management API to be ready..."

sleep 3

docker exec rabbitmq rabbitmqctl status > /dev/null

# Source configuration file to load variables
source rabbit-infrastructure.cfg

echo "Declaring queue '${PROD_TO_BALANCER_QUEUE_NAME}' with options 'durable=false'"
docker exec ${RABBITMQ_CONTAINER_NAME} rabbitmqadmin declare queue name="${PROD_TO_BALANCER_QUEUE_NAME}" durable=false
