use crate::common::*;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct KafkaBroker {
    pub brokers: String,
    
}


impl KafkaBroker { 
    
    /*
        Function that creates a Producer object
    */
    pub fn create_producer(&self) -> Result<FutureProducer, anyhow::Error> {
        
        let producer:FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .create()?;
        
        Ok(producer)
    }
}
