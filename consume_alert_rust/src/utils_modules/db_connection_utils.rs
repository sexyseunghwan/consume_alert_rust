use crate::common::*;

use crate::repository::kafka_repository::*;
use crate::repository::es_repository::*;

/*
    Function that initializes db connection to a 'single tone'
*/
pub async fn initialize_db_clients() {
    
    dotenv().ok();

    let es_host: Vec<String> = env::var("ES_DB_URL").expect("[ENV file read Error][initialize_db_clients()] 'ES_DB_URL' must be set").split(',').map(|s| s.to_string()).collect();
    let es_id = env::var("ES_ID").expect("[ENV file read Error][initialize_db_clients()] 'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("[ENV file read Error][initialize_db_clients()] 'ES_PW' must be set");
    
    // Elasticsearch connection
    let es_client: EsRepositoryPub = match EsRepositoryPub::new(es_host, &es_id, &es_pw) {
        Ok(es_client) => es_client,
        Err(err) => {
            error!("[DB Connection Error][initialize_db_clients()] Failed to create Elasticsearch client : {:?}", err);
            panic!("[DB Connection Error][initialize_db_clients()] Failed to create Elasticsearch client : {:?}", err);
        }
    };
    
    let _ = ELASTICSEARCH_CLIENT.set(Arc::new(es_client));

    let kafka_host: String = env::var("KAFKA_HOST").expect("[ENV file read Error][initialize_db_clients()] 'KAFKA_HOST' must be set");

    // Kafka connection
    let kafka_produce_broker: KafkaRepositoryPub = match KafkaRepositoryPub::new(&kafka_host) {
        Ok(kafka_client) => kafka_client,
        Err(err) => {
            error!("[DB Connection Error][initialize_db_clients()] Failed to create Kafka client: {:?}", err);
            panic!("[DB Connection Error][initialize_db_clients()] Failed to create Kafka client: {:?}", err)
        }
    };

    let _ = KAFKA_PRODUCER.set(Arc::new(kafka_produce_broker));


    // MySQL connection
        
}