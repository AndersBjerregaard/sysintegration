Run the application with `cargo run`, with the working directory being `resequencer-app`.

Publish messages to the queue `q_resequencer`, ideally routed from the default exchange.

Attach 3 headers to the message:
- `sequence_id` with a String value
- `sequence` with a Number value
- `num_sequences` with a Number value

A practical example would entail publishing 3 messages with the same `sequence_id` and `num_sequences` but in an unordered fashion. For example with the values for the `sequence` key being *2*, *3*, and *1*. And with some meaningful payload that you'd like in an ordered fashion. This would showcase that the resequencer will wait for all messages with a corresponding `sequence_id` to be consumed, before *resequencing* them in an ordered fashion. No matter in what order they were consumed in.