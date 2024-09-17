use crate::common::*;

use crate::service::es_service::*;
use crate::service::kafka_service::*;

/*
    Function that initializes db connection to a 'single tone'
*/
pub async fn initialize_db_clients() {
    
    dotenv().ok();

    let es_host: Vec<String> = env::var("ES_DB_URL").expect("[ENV file read Error] 'ES_DB_URL' must be set").split(',').map(|s| s.to_string()).collect();
    let es_id = env::var("ES_ID").expect("[ENV file read Error] 'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("[ENV file read Error] s'ES_PW' must be set");

    let kafka_host: String = env::var("KAFKA_HOST").expect("[ENV file read Error] 'KAFKA_HOST' must be set");
    
    // Elasticsearch connection
    let es_client: EsHelper = match EsHelper::new(es_host, &es_id, &es_pw) {
        Ok(es_client) => es_client,
        Err(err) => {
            error!("[DB Connection Error] Failed to create mysql client - main_controller() // {:?}", err);
            panic!("[DB Connection Error] Failed to create mysql client - main_controller() // {:?}", err);
        }
    };

    let _ = ELASTICSEARCH_CLIENT.set(Arc::new(es_client));
    
    // Kafka connection
    let kafka_produce_broker: ProduceBroker = match ProduceBroker::new(&kafka_host) {
        Ok(kafka_client) => kafka_client,
        Err(err) => {
            error!("Failed to create Kafka client: {:?}", err);
            panic!("Failed to create Kafka client: {:?}", err)
        }
    };

    let _ = KAFKA_PRODUCER.set(Arc::new(kafka_produce_broker));
    
}