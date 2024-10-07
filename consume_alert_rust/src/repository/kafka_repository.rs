use crate::common::*;

#[async_trait]
pub trait KafkaRepository {
    async fn produce_message(&self, topic: &str, message: &str) -> Result<(), anyhow::Error>;
    async fn logging_kafka(&self, msg: &str);
}

pub struct KafkaRepositoryPub {
    produce_broker: FutureProducer
}

impl KafkaRepositoryPub {
    
    /*
        Constructor of Kafka Producer
    */
    pub fn new(kafka_host: &str) -> Result<Self, anyhow::Error> {

        let kafka_producer:FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", kafka_host)
            .create()?;
        
        let produce_client = KafkaRepositoryPub {
            produce_broker: kafka_producer
        };
        
        Ok(produce_client)
    }
}


#[async_trait]
impl KafkaRepository for KafkaRepositoryPub { 
    

    /* 
        Kafka Function that produces messages on a specific topic
    */
    async fn produce_message(&self, topic: &str, message: &str) -> Result<(), anyhow::Error>  {
    
        let kafka_producer = &self.produce_broker;
        
        let record = FutureRecord::to(topic)
            .payload(message)
            .key("");  // You can set a key for the message if needed
        
        match kafka_producer.send(record, Duration::from_secs(5)).await {
            Ok(_) => { Ok(()) },
            Err((e, _)) => Err(anyhow!(e.to_string())),
        }
    }


    /*
        Function that SENDS the entire log of the program to KAFKA
    */
    async fn logging_kafka(&self, msg: &str) {
        
        let _ = match self.produce_message("consume_alert_rust", msg).await {
            Ok(_) => (),
            Err(e) => {
                error!("{:?}", e)
            }
        };
    }  
    
}
