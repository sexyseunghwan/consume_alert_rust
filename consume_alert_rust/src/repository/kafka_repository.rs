use crate::common::*;

#[async_trait]
pub trait KafkaRepository {
    /// Serializes a JSON payload and sends it as a Kafka message to the specified topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - The Kafka topic name to send the message to
    /// * `key` - Optional partition key for the message
    /// * `payload` - The JSON value to serialize and send as the message body
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the Kafka delivery fails.
    async fn send_message(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &Value,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Getters, Clone)]
pub struct KafkaRepositoryImpl {
    producer: FutureProducer,
}

impl KafkaRepositoryImpl {
    /// Creates a new `KafkaRepositoryImpl` by reading broker and security settings from environment variables.
    ///
    /// # Returns
    ///
    /// Returns `Ok(KafkaRepositoryImpl)` on successful producer creation.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing or the Kafka producer cannot be created.
    pub fn new() -> anyhow::Result<Self> {
        let kafka_brokers: String = env::var("KAFKA_BROKERS").inspect_err(|e| {
            error!(
                "[KafkaRepositoryImpl::new] 'KAFKA_BROKERS' must be set: {:#}",
                e
            );
        })?;

        let security_protocol: String =
            env::var("KAFKA_SECURITY_PROTOCOL").unwrap_or_else(|_| "PLAINTEXT".to_string());

        let mut config: ClientConfig = ClientConfig::new();
        config
            .set("bootstrap.servers", &kafka_brokers)
            .set("message.timeout.ms", "30000")
            .set("queue.buffering.max.messages", "100000")
            .set("queue.buffering.max.kbytes", "1048576")
            .set("batch.num.messages", "10000")
            .set("security.protocol", &security_protocol);

        if security_protocol.contains("SASL") {
            let sasl_mechanism: String = env::var("KAFKA_SASL_MECHANISM").expect(
                "[KafkaRepositoryImpl::new] 'KAFKA_SASL_MECHANISM' must be set when using SASL",
            );
            let sasl_username: String = env::var("KAFKA_SASL_USERNAME").expect(
                "[KafkaRepositoryImpl::new] 'KAFKA_SASL_USERNAME' must be set when using SASL",
            );
            let sasl_password: String = env::var("KAFKA_SASL_PASSWORD").expect(
                "[KafkaRepositoryImpl::new] 'KAFKA_SASL_PASSWORD' must be set when using SASL",
            );

            config
                .set("sasl.mechanism", &sasl_mechanism)
                .set("sasl.username", &sasl_username)
                .set("sasl.password", &sasl_password);
        }

        let producer: FutureProducer = config.create().map_err(|e| {
            anyhow!(
                "[KafkaRepositoryImpl::new] Failed to create producer: {:?}",
                e
            )
        })?;

        Ok(KafkaRepositoryImpl { producer })
    }
}

#[async_trait]
impl KafkaRepository for KafkaRepositoryImpl {
    #[doc = "Function that sends JSON message to Kafka topic"]
    async fn send_message(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &Value,
    ) -> anyhow::Result<()> {
        let payload_str: String = serde_json::to_string(payload).map_err(|e| {
            anyhow!(
                "[KafkaRepositoryImpl::send_message] Failed to serialize payload: {:?}",
                e
            )
        })?;

        let mut record: FutureRecord<'_, str, String> =
            FutureRecord::to(topic).payload(&payload_str);

        if let Some(k) = key {
            record = record.key(k);
        }

        match self.producer.send(record, Duration::from_secs(30)).await {
            Ok(delivery) => {
                info!(
                    "[KafkaRepositoryImpl::send_message] Message sent to topic: {}, partition: {}, offset: {}",
                    topic, delivery.partition, delivery.offset
                );
                Ok(())
            }
            Err((e, _)) => {
                let error_message: String = format!(
                    "[KafkaRepositoryImpl::send_message] Failed to send message to topic {}: {:?}",
                    topic, e
                );
                Err(anyhow!(error_message))
            }
        }
    }
}
