use crate::common::*;

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ConsumeIndexProd {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub prodt_money: i64,
    pub prodt_name: String,
    pub prodt_type: Option<String>
}