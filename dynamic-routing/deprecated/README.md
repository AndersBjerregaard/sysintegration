# The Idea

Have the **Consumer** application scale horizontally, and flexibly attach itself to the **Load Balancer** at runtime. The **Load Balancer** dynamically routes messages from the **producer** to a **consumer** that is ready to process the message. Should the **producer** push out more messages than there are available consumers to process the given message, the message should be rejected and dead-lettered: Whereas it should be re-queued to the workload queue in an attempt for it to be processed again. Hereby there should be some logic to track how many retries a given message has had, and fully discard the message if it has reached a certain threshold.



## The Issue At The Moment

Sharing a single instance of information across multiple threads at the same time. Currently I'm attempting to use an `Arc<Mutex<T>>` struct type to use atomical reference across multiple threads. But doesn't seem to work as intended with the `set_delegate` function from the `lapin::consumer` struct.