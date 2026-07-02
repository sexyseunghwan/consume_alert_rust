use crate::common::*;

use crate::service_traits::mysql_query_service::*;

use crate::models::currency_exchange_rate_snapshot::*;

#[doc = "Function that determines if the string consists of only numbers"]
#[allow(dead_code)]
pub fn is_numeric(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

#[doc = "Functions that convert strings into numbers"]
#[allow(dead_code)]
pub fn to_numeric(s: &str) -> i64 {
    s.parse::<i64>().unwrap_or(0)
}

#[doc = "Formats a Decimal with thousand separators on the integer part, keeping N decimal places"]
pub fn format_decimal_with_commas(value: Decimal, decimals: u32) -> String {
    let sign: &str = if value.is_sign_negative() { "-" } else { "+" };
    let rounded: Decimal = value.abs().round_dp(decimals);
    let formatted: String = format!("{:.*}", decimals as usize, rounded);
    let (int_part, frac_part) = formatted.split_once('.').unwrap_or((&formatted, ""));
    let int_formatted: String = int_part
        .parse::<i64>()
        .unwrap_or(0)
        .to_formatted_string(&Locale::en);

    if decimals > 0 {
        format!("{}{}.{}", sign, int_formatted, frac_part)
    } else {
        format!("{}{}", sign, int_formatted)
    }
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
