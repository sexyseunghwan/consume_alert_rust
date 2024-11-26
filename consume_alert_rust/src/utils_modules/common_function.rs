use crate::common::*;

use crate::repository::es_repository::*;
use crate::repository::kafka_repository::*;


#[doc = "Function to initialize Database connection instances"]
pub fn initialize_db_connection() {
    initialize_elastic_clients();
    initialize_kafka_clients();
}


#[doc = "Function that takes a particular value from a vector - Access by Index"]
/// # Arguments
/// * `vec`     - Vector data
/// * `index`   - Parameters for which number of elements to be drawn from the vector
/// 
/// # Returns
/// * Result<T, anyhow::Error>
pub fn get_parsed_value_from_vector<T: FromStr>(vec: &Vec<String>, index: usize) -> Result<T, anyhow::Error>
where
    T::Err: std::fmt::Debug,
{
    let value_str = vec.get(index)
        .ok_or_else(|| anyhow!("[Index Out Of Range Error] The {}th element does not exist.", index))?;
    
    value_str.parse::<T>()
        .map_err(|e| anyhow!("[Parse Error] Failed to parse the value at index {}: {:?}", index, e))
}