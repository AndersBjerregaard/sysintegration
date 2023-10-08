FROM rabbitmq:3-management

# Expose management and amqp protocol port
EXPOSE 5672 15672

# Copy a script that will be executed when the container starts
COPY init.sh /init.sh

# Make the script executable
RUN chmod +x /init.sh

# Set the CMD instruction to run the script
CMD ["/init.sh"]
