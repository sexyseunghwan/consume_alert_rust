use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, new)]
#[getset(get = "pub")]
pub struct StockPieDataDto {
    pub stock_alias: String,
    pub stock_amount_krw: Decimal,
}
