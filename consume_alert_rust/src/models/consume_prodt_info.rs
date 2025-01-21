use crate::common::*;

#[doc = "consume_index_prodt 인덱스와 맵핑되는 구조체 - 소비정보가 담겨있는 구조체정보"]
#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct ConsumeProdtInfo {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub cur_timestamp: String,
    pub prodt_name: String,
    pub prodt_money: i64,
    pub prodt_type: String,
}
