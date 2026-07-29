use crate::common::*;

use crate::dtos::StockPieDataDto;
use crate::models::stock_resp::StockRespDetail;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, new)]
#[getset(get = "pub")]
pub struct StockPieData {
    pub stock_names: Vec<String>,
    pub stock_amount_krw: Vec<Decimal>,
    pub total_stock_amount_krw: Decimal,
}

/// Buckets small stock positions (<= threshold weight) into a single "ETC" slice for the pie chart.
pub fn build_stock_pie_data(
    stock_resp_details: &[StockRespDetail],
    total_stock_amount_krw: Decimal,
) -> StockPieData {
    let etc_threshold: Decimal = Decimal::new(3, 2);
    let mut stock_pie_data_dtos: Vec<StockPieDataDto> = Vec::new();
    let mut etc_amount_krw: Decimal = Decimal::ZERO;

    for resp in stock_resp_details {
        if resp.stock_portfolio_weight <= etc_threshold {
            etc_amount_krw += *resp.stock_total_price_krw();
        } else {
            stock_pie_data_dtos.push(StockPieDataDto {
                stock_alias: resp.stock_alias.clone(),
                stock_amount_krw: *resp.stock_total_price_krw(),
            });
        }
    }

    if etc_amount_krw != Decimal::ZERO {
        stock_pie_data_dtos.push(StockPieDataDto {
            stock_alias: "ETC".to_string(),
            stock_amount_krw: etc_amount_krw,
        });
    }

    StockPieData::new(
        stock_pie_data_dtos
            .iter()
            .map(|s| s.stock_alias().to_string())
            .collect(),
        stock_pie_data_dtos
            .iter()
            .map(|s| *s.stock_amount_krw())
            .collect(),
        total_stock_amount_krw,
    )
}
