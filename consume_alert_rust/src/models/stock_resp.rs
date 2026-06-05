use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct StockResp {
    pub stock_name: String,
    pub stock_total_price: Decimal,
}
