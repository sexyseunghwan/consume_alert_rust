use crate::common::*;

use crate::models::consume_prodt_info::*;

#[doc = "Structure containing consumption information. - includes installment information"]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct ConsumeProdtInfoByInstallment {
    pub installment: i64,
    pub consume_prodt_info: ConsumeProdtInfo,
}
