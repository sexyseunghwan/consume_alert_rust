use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, new)]
#[getset(get = "pub")]
pub struct StockPieData {
    pub stock_names: Vec<String>,
    pub stock_amount_krw: Vec<Decimal>,
    pub total_stock_amount_krw: Decimal,
}