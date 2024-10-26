use crate::common::*;

use crate::repository::es_repository::*;
use crate::repository::kafka_repository::*;


#[doc = "Function to initialize Database connection instances"]
pub fn initialize_db_connection() {
    initialize_elastic_clients();
    initialize_kafka_clients();
}