use crate::common::*;

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ConsumeTypeInfo {
    pub prodt_type: String,
    pub prodt_cost: i32,
    pub prodt_per: f64,
}
