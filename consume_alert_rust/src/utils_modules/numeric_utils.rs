use crate::common::*;

use crate::service_traits::mysql_query_service::*;

use crate::models::currency_exchange_rate_snapshot::*;

#[doc = "Function that determines if the string consists of only numbers"]
pub fn is_numeric(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

#[doc = "Functions that convert strings into numbers"]
pub fn to_numeric(s: &str) -> i64 {
    s.parse::<i64>().unwrap_or(0)
}

pub async fn fetch_exchange_rate<M>(
    mysql_query_service: &M,
    base_currency_code: &str,
    target_currency_code: &str,
) -> anyhow::Result<Decimal>
where
    M: MysqlQueryService,
{
    let currencies: Vec<CurrencyExchangeRateSnapshot> = mysql_query_service
        .find_currency_exchange_rate_snapshot(base_currency_code, target_currency_code)
        .await?;

    let currency: &CurrencyExchangeRateSnapshot = currencies.first().ok_or_else(|| {
        anyhow!(
            "[numeric_utils::fetch_exchange_rate] Exchange rate not found. base={}, target={}",
            base_currency_code,
            target_currency_code
        )
    })?;

    Ok(*currency.exchange_rate())
}
