use crate::common::*;

#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct ConsumingIndexProdType {
    pub consume_keyword_type: String,
    pub consume_keyword: String,
}
