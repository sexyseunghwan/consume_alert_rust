use crate::common::*;

#[async_trait]
pub trait ProducerService {
    /// Serializes a single object to JSON and produces it as a message to the specified Kafka topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - The Kafka topic name to send the message to
    /// * `object` - The serializable object to send
    /// * `key` - Optional partition key for the Kafka message
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the Kafka send fails.
    async fn produce_object_to_topic<T>(
        &self,
        topic: &str,
        object: &T,
        key: Option<&str>,
    ) -> Result<(), anyhow::Error>
    where
        T: Serialize + Send + Sync;

    #[allow(dead_code)]
    /// Serializes multiple objects to JSON and produces each as a separate message to the specified Kafka topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - The Kafka topic name to send messages to
    /// * `objects` - Slice of serializable objects to send
    /// * `key_fn` - Optional function to generate a partition key from each object
    ///
    /// # Errors
    ///
    /// Returns an error if any serialization or Kafka send fails.
    async fn produce_objects_to_topic<T, F>(
        &self,
        topic: &str,
        objects: &[T],
        key_fn: Option<F>,
    ) -> Result<(), anyhow::Error>
    where
        T: Serialize + Send + Sync,
        F: Fn(&T) -> String + Send + Sync + 'static;
}
