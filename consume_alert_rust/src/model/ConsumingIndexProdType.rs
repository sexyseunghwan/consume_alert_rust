use crate::common::*;


#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct ConsumingIndexProdType {
    keyword_type: String,
    keyword: String,
    bias_value: i32
}