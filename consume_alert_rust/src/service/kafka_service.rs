use crate::common::*;

use crate::repository::kafka_repository::*;

#[derive(Clone)]
pub struct ProduceBroker {
    produce_broker: FutureProducer
}


impl ProduceBroker {
    
    /*
        Constructor of Kafka Producer
    */
    pub fn new(kafka_host: &str) -> Result<Self, anyhow::Error> {
        
        let kafka_client = KafkaBroker {
            brokers: kafka_host.to_string()
        };
        
        let kafka_producer = kafka_client.create_producer()?;
        
        let produce_client = ProduceBroker {
            produce_broker: kafka_producer
        };
        
        Ok(produce_client)
    }
    
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
    pub async fn logging_kafka(&self, msg: &str) {
        
        let _ = match self.produce_message("consume_alert_rust", msg).await {
            Ok(_) => (),
            Err(e) => {
                error!("{:?}", e)
            }
        };
    }        
}



