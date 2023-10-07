#!/bin/bash

# Start RabbitMQ server in the background
rabbitmq-server -detached

# Function to check if RabbitMQ management API is ready
rabbitmq_ready() {
    docker exec "$RABBITMQ_CONTAINER_NAME" rabbitmqctl status > /dev/null 2>&1
}

# Wait for RabbitMQ management API to be ready
WAIT_TIMEOUT=300  # Adjust the timeout according to your needs
WAIT_INTERVAL=5   # Adjust the interval between checks

echo "Waiting for RabbitMQ management API to be ready..."
elapsed_time=0

while ! rabbitmq_ready; do
    if [ $elapsed_time -ge $WAIT_TIMEOUT ]; then
        echo "Timeout reached. RabbitMQ management API not ready."
        exit 1
    fi

    sleep $WAIT_INTERVAL
    elapsed_time=$((elapsed_time + WAIT_INTERVAL))
done

# RabbitMQ management API is ready, execute your rabbitmqadmin commands
echo "RabbitMQ management API is ready. Executing rabbitmqadmin commands..."
# Your rabbitmqadmin commands go here

echo "Setup complete."

# Keep the container running
tail -f /dev/null
