use crate::common::*;

#[doc = "Structure containing consumption information."]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct ConsumeProdtInfo {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub cur_timestamp: String,
    pub prodt_name: String,
    pub prodt_money: i64,
    pub prodt_type: String,
}
