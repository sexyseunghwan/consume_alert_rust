use crate::common::*;

use crate::models::asset_collection::{ExchangeRates, SEP};
use crate::utils_modules::numeric_utils::format_decimal_with_commas;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct StockResp {
    pub stock_seq: i64,
    pub stock_name: String,
    pub stock_alias: String,
    pub stock_price: Decimal,
    pub stock_cnt: i64,
    pub avg_purchase_price: Decimal,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters)]
#[getset(get = "pub")]
pub struct StockRespDetail {
    pub stock_name: String,
    pub stock_alias: String,
    pub stock_total_price_usd: Decimal,
    pub stock_total_price_krw: Decimal,
    pub stock_roi: Decimal,
    pub stock_invest_profit_usd: Decimal,
    pub stock_invest_profit_krw: Decimal,
    pub stock_portfolio_weight: Decimal,
    pub avg_purchase_price_krw: Decimal,
}

impl StockResp {
    pub fn convert_to_stock_resp_detail(
        &self,
        total_stock_amount_krw: Decimal,
        currency_code: String,
        usd_to_krw: Decimal,
        krw_to_usd: Decimal,
    ) -> StockRespDetail {
        let stock_cnt: Decimal = Decimal::from(self.stock_cnt);
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

        let avg_purchase_price_krw: Decimal = if self.avg_purchase_price.is_zero() {
            Decimal::ZERO
        } else {
            let base_profit: Decimal = self.avg_purchase_price * stock_cnt;
            if is_krw {
                base_profit
            } else {
                base_profit * usd_to_krw
            }
        };

        StockRespDetail {
            stock_name: self.stock_name().to_string(),
            stock_alias: self.stock_alias().to_string(),
            stock_total_price_usd,
            stock_total_price_krw,
            stock_roi,
            stock_invest_profit_usd,
            stock_invest_profit_krw,
            stock_portfolio_weight,
            avg_purchase_price_krw,
        }
    }
}

/// Converts the raw stock holdings into display-ready details, sorted by KRW value descending.
pub fn build_stock_details(
    stock_list: &[StockResp],
    total_stock_amount_krw: Decimal,
    usd_to_krw: Decimal,
    krw_to_usd: Decimal,
) -> Vec<StockRespDetail> {
    let mut stock_resp_details: Vec<StockRespDetail> = stock_list
        .iter()
        .map(|stock| {
            stock.convert_to_stock_resp_detail(
                total_stock_amount_krw,
                stock.currency_code().to_string(),
                usd_to_krw,
                krw_to_usd,
            )
        })
        .collect();

    stock_resp_details.sort_by_key(|s| std::cmp::Reverse(s.stock_total_price_krw));

    stock_resp_details
}

/// Builds the Telegram summary message for the stock portfolio.
pub fn build_stock_message(
    stock_resp_details: &[StockRespDetail],
    total_stock_amount_krw: Decimal,
    stock_avg_purchase_price_krw: Decimal,
    rates: ExchangeRates,
) -> String {
    let total_stock_amount_usd: Decimal = total_stock_amount_krw * rates.krw_to_usd;

    let mut msg: String = format!("{}\n[주식 포트폴리오]\n", SEP);

    if stock_resp_details.is_empty() {
        msg.push_str("  (없음)\n");
    } else {
        for stock in stock_resp_details {
            msg.push_str(&format!(
                "*  {} : \n      {}₩ ({:.2}$) \n            ROI: {:.3}%\n            PROFIT(₩): {}\n",
                stock.stock_alias(),
                format_decimal_with_commas(stock.stock_total_price_krw, 0, false),
                stock.stock_total_price_usd.round_dp(2),
                stock.stock_roi,
                format_decimal_with_commas(stock.stock_invest_profit_krw, 0, true)
            ));
        }
    }

    let total_stock_profit: Decimal = total_stock_amount_krw - stock_avg_purchase_price_krw;
    let total_stock_roi: Decimal =
        total_stock_profit / stock_avg_purchase_price_krw * Decimal::from(100);

    msg.push_str(&format!(
        "{}\n총 주식: \n      {}₩ ({:.2}$)\n            ROI: {:.3}%\n            PROFIT(₩): {}\n",
        SEP,
        format_decimal_with_commas(total_stock_amount_krw, 0, false),
        total_stock_amount_usd.round_dp(2),
        total_stock_roi.round_dp(2),
        format_decimal_with_commas(total_stock_profit, 0, true)
    ));

    msg.push_str(SEP);
    msg
}
