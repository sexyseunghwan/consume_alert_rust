use crate::common::*;

#[derive(Debug, Getters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct DistinctObject {
    key: String,
    doc_count: i64,
}
