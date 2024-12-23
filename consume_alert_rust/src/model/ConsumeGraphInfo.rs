use crate::common::*;

use crate::model::ConsumeIndexProdNew::*;


#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct ConsumeGraphInfo {
    total_consume_pay: f64,
    consume_list: Vec<ConsumeIndexProdNew>,
    start_dt: NaiveDate,
    end_dt: NaiveDate
}