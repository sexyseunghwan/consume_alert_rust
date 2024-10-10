use crate::common::*;

// #[async_trait]
// pub trait KafkaRepository {
//     async fn produce_message(&self, topic: &str, message: &str) -> Result<(), anyhow::Error>;
//     async fn logging_kafka(&self, msg: &str);
// }

// pub struct KafkaRepositoryPub {
//     produce_broker: FutureProducer
// }

// impl KafkaRepositoryPub {
    
//     /*
//         Constructor of Kafka Producer
//     */
//     pub fn new(kafka_host: &str) -> Result<Self, anyhow::Error> {

//         let kafka_producer:FutureProducer = ClientConfig::new()
//             .set("bootstrap.servers", kafka_host)
//             .create()?;
        
//         let produce_client = KafkaRepositoryPub {
//             produce_broker: kafka_producer
//         };
        
//         Ok(produce_client)
//     }
// }


// #[async_trait]
// impl KafkaRepository for KafkaRepositoryPub { 
    

//     /* 
//         Kafka Function that produces messages on a specific topic
//     */
//     async fn produce_message(&self, topic: &str, message: &str) -> Result<(), anyhow::Error>  {
    
//         let kafka_producer = &self.produce_broker;
        
//         let record = FutureRecord::to(topic)
//             .payload(message)
//             .key("");  // You can set a key for the message if needed
        
//         match kafka_producer.send(record, Duration::from_secs(5)).await {
//             Ok(_) => { Ok(()) },
//             Err((e, _)) => Err(anyhow!(e.to_string())),
//         }
//     }


//     /*
//         Function that SENDS the entire log of the program to KAFKA
//     */
//     async fn logging_kafka(&self, msg: &str) {
        
//         let _ = match self.produce_message("consume_alert_rust", msg).await {
//             Ok(_) => (),
//             Err(e) => {
//                 error!("{:?}", e)
//             }
//         };
//     }  
    
// }

/*

*/
static KAFKA_PRODUCER: once_lazy<Arc<KafkaRepositoryPub>> = once_lazy::new(|| {
    initialize_kafka_clients()
});


/*

*/
pub fn initialize_kafka_clients() -> Arc<KafkaRepositoryPub> {

    let kafka_host: String = env::var("KAFKA_HOST").expect("[ENV file read Error][initialize_db_clients()] 'KAFKA_HOST' must be set");
    let kafka_host_vec: Vec<String> = kafka_host.split(',')
        .map(|s| s.to_string())
        .collect();

    let produce_broker: Producer = match Producer::from_hosts(kafka_host_vec.to_owned())
        .with_ack_timeout(Duration::from_secs(3)) // Timeout settings for message transfer confirmation
        .with_required_acks(RequiredAcks::One)// If the message transfer is delivered to at least one broker, it is considered a success
        .create() {
            Ok(kafka_producer) => kafka_producer,
            Err(e) => {
                error!("{:?}", e);
                panic!("{:?}", e)
            }
        };
    
    let kafka_producer = KafkaRepositoryPub::new(produce_broker);
    Arc::new(kafka_producer)
}

#[async_trait]
pub trait KafkaRepository {
    fn produce_message(&self, topic: &str, message: &str) -> Result<(), anyhow::Error>;
    async fn logging_kafka(&self, msg: &str);
}

#[derive(new)]
pub struct KafkaRepositoryPub {
    produce_broker: Producer
}

#[async_trait]
impl KafkaRepository for KafkaRepositoryPub {

    /* 

    */
    fn produce_message(&self, topic: &str, message: &str) -> Result<(), anyhow::Error> {

        let produce_broker = self.produce_broker;
        let result = produce_broker.send(&KafkaRecord::from_value(topic, message))?;
        
        Ok(())
    }
    
    
    /*

    */
    async fn logging_kafka(&self, msg: &str) {

        let handle = task::spawn_blocking(move || {
            self.produce_message("consume_alert_rust", msg);
        });
        
        let res = handle.await;
        
    }


}
