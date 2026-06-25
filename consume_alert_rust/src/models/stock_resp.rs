use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct StockResp {
    pub stock_seq: i64,
    pub stock_name: String,
    pub stock_price: Decimal,
    pub stock_cnt: i64,
    pub avg_purchase_price: Decimal,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct StockRespDetail {
    pub stock_name: String,
    pub stock_total_price_usd: Decimal,
    pub stock_total_price_krw: Decimal,
    pub stock_roi: Decimal,
    pub stock_invest_profit_usd: Decimal,
    pub stock_invest_profit_krw: Decimal,
    pub stock_portfolio_weight: Decimal,
}

impl StockResp {
    pub fn convert_to_stock_resp_detail(
        &self,
        total_stock_amount_krw: Decimal,
        currency_code: String,
        usd_to_krw: Decimal,
        krw_to_usd: Decimal,
    ) -> StockRespDetail {
        let stock_cnt = Decimal::from(self.stock_cnt);
        let stock_total_price: Decimal = stock_cnt * self.stock_price;

        let is_krw: bool = currency_code == "KRW";

        let (stock_total_price_usd, stock_total_price_krw) = if is_krw {
            (stock_total_price * krw_to_usd, stock_total_price)
        } else {
            (stock_total_price, stock_total_price * usd_to_krw)
        };

        let stock_roi: Decimal = if self.avg_purchase_price.is_zero() {
            Decimal::ZERO
        } else {
            ((self.stock_price - self.avg_purchase_price) / self.avg_purchase_price
                * Decimal::from(100))
            .round_dp(3)
        };

        let (stock_invest_profit_usd, stock_invest_profit_krw) = if self.stock_cnt == 0 {
            (Decimal::ZERO, Decimal::ZERO)
        } else {
            let base_profit: Decimal =
                ((self.stock_price - self.avg_purchase_price) * stock_cnt).round_dp(3);
            if is_krw {
                ((base_profit * krw_to_usd).round_dp(3), base_profit)
            } else {
                (base_profit, (base_profit * usd_to_krw).round_dp(3))
            }
        };

        let stock_portfolio_weight: Decimal = if total_stock_amount_krw.is_zero() {
            Decimal::ZERO
        } else {
            (stock_total_price_krw / total_stock_amount_krw).round_dp(3)
        };

        StockRespDetail::new(
            self.stock_name().to_string(),
            stock_total_price_usd,
            stock_total_price_krw,
            stock_roi,
            stock_invest_profit_usd,
            stock_invest_profit_krw,
            stock_portfolio_weight,
        )
    }
}
