use crate::common::*;

#[doc = ""]
#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ConsumeResultByType {
    pub consume_prodt_type: String,
    pub consume_prodt_per: f64,
}