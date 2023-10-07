FROM rabbitmq:3-management

# Expose management and amqp protocol port
EXPOSE 5672 15672

RUN rabbitmqadmin declare exchange name=my-new-exchange type=fanout
