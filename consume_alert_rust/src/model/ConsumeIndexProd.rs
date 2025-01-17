use crate::common::*;

use crate::model::ConsumingIndexProdType::*;

#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct ConsumeIndexProd {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub prodt_money: i64,
    pub prodt_name: String,
    pub prodt_type: Option<String>,
    pub prodt_type_query_res: Option<Vec<ConsumingIndexProdType>>,
}
