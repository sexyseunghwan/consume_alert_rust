use crate::common::*;

#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct ConsumingIndexProdtType {
    pub consume_keyword_type: String,
    pub consume_keyword: String,
}
