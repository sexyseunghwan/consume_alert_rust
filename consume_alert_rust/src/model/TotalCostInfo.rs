use crate::common::*;
use crate::model::ConsumeIndexProd::*;

#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct TotalCostInfo {
    pub total_cost: f64,
    pub consume_list: Vec<ConsumeIndexProd>,
    pub empty_flag: bool,
    pub start_dt: NaiveDate,
    pub end_dt: NaiveDate,
}
