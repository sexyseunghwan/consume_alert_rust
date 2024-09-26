use crate::common::*;

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ProdtDetailInfo {
    pub keyword: String,
    pub bias_value: i32
}