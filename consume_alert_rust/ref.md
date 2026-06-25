use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct StockResp {
    pub stock_seq: i64,
    pub stock_name: String,
    pub stock_price: Decimal,
    pub stock_cnt: Decimal,
    pub avg_purchase_price: Decimal,
    pub currency_code: String
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
    pub fn convert_to_stock_resp_detail(&self, total_stock_amount_krw: Decimal, is_krw: bool, usd_to_krw: Decimal, krw_to_usd: Decimal) -> StockRespDetail {
        let stock_name: String = self.stock_name().to_string();
        let stock_total_price: Decimal = self.stock_cnt * self.stock_price;
        
        let (stock_total_price_usd, stock_total_price_krw) = if is_krw {
            (stock_total_price*krw_to_usd, stock_total_price)
        } else {
            (stock_total_price, stock_total_price*usd_to_krw)
        };
        
        
        /*
            집계는 원화 기준으로 한다...
        */ 
        let stock_roi: Decimal = if self.avg_purchase_price.is_zero() {
            Decimal::ZERO
        } else {
            ((self.stock_price - self.avg_purchase_price) / self.avg_purchase_price
                * Decimal::from(100))
            .round_dp(3)
        };

        let (stock_invest_profit_usd, stock_invest_profit_krw) = if is_krw {
            if self.stock_cnt.is_zero() {
                (Decimal::ZERO, Decimal::ZERO)
            } else {
                let stock_invest_profit_krw: Decimal = 
                    ((self.stock_price * self.stock_cnt) - (self.avg_purchase_price * self.stock_cnt) / Decimal::from(100)).round_dp(3);                
                let stock_invest_profit_usd: Decimal = stock_invest_profit_krw * krw_to_usd;
                (stock_invest_profit_usd, stock_invest_profit_krw)
            }
        } else {
            if self.stock_cnt.is_zero() {
                (Decimal::ZERO, Decimal::ZERO)
            } else {
                let stock_invest_profit_usd: Decimal = 
                    ((self.stock_price * self.stock_cnt) - (self.avg_purchase_price * self.stock_cnt) / Decimal::from(100)).round_dp(3);
                let stock_invest_profit_krw: Decimal = stock_invest_profit_usd * usd_to_krw;
                (stock_invest_profit_usd, stock_invest_profit_krw)
            }
        };

        let stock_portfolio_weight = if total_stock_amount_krw.is_zero() {
            Decimal::ZERO
        } else {
            ((self.stock_cnt * stock_total_price_krw) / total_stock_amount_krw).round_dp(3)
        };

        StockRespDetail::new(
            stock_name, 
            stock_total_price_usd, 
            stock_total_price_krw, 
            stock_roi, 
            stock_invest_profit_usd, 
            stock_invest_profit_krw, 
            stock_portfolio_weight
        )
    }
}
