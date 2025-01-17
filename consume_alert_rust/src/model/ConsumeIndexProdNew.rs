use crate::common::*;

use crate::model::ConsumingIndexProdType::*;

#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct ConsumeIndexProdNew {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub cur_timestamp: String,
    pub prodt_name: String,
    pub prodt_money: i64,
    pub prodt_type: String,
}
