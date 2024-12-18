use crate::common::*;


#[doc = "Function to convert structure to JSON value"]
pub fn convert_json_from_struct<T: Serialize>(input_struct: T) -> Result<Value, anyhow::Error> {
    
    serde_json::to_value(input_struct)
        .map_err(|err| anyhow!("[Error][convert_json_from_struct()] Failed to serialize struct to JSON: {}", err))
    
}