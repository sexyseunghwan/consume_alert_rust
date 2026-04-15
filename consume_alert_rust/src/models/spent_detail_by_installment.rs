use crate::common::*;

use super::spent_detail::*;

#[allow(dead_code)]
#[doc = "Structure containing spent detail with installment information."]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, new)]
#[getset(get = "pub")]
pub struct SpentDetailByInstallment {
    pub installment: i64,
    pub spent_detail: SpentDetail,
}
