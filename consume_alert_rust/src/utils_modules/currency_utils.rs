use crate::common::*;

static HTTP_CLIENT: once_lazy<Client> = once_lazy::new(|| reqwest::Client::new());

const TWELVE_DATA_URL: &str = "https://api.twelvedata.com/exchange_rate?symbol=USD%2FKRW&apikey=";

#[doc = "Function to fetch the real-time USD -> KRW exchange rate via Twelve Data"]
/// # Returns
/// * `Result<f64, anyhow::Error>` - exchange rate (1 USD = N KRW)
pub async fn get_usd_to_krw_rate() -> anyhow::Result<f64> {
    let api_key: String = env::var("TWELVE_DATA_API_KEY").map_err(|e| {
        anyhow!("[get_usd_to_krw_rate] 'TWELVE_DATA_API_KEY' must be set: {:#}", e)
    })?;

    let url: String = format!("{}{}", TWELVE_DATA_URL, api_key);

    let response: Value = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("[get_usd_to_krw_rate] HTTP request failed: {:#}", e))?
        .json::<Value>()
        .await
        .map_err(|e| anyhow!("[get_usd_to_krw_rate] Failed to parse JSON response: {:#}", e))?;

    if let Some(code) = response["code"].as_u64() {
        return Err(anyhow!(
            "[get_usd_to_krw_rate] Twelve Data API error (code {}): {}",
            code,
            response["message"].as_str().unwrap_or("unknown")
        ));
    }

    let rate: f64 = response["rate"]
        .as_f64()
        .ok_or_else(|| anyhow!("[get_usd_to_krw_rate] 'rate' field missing in response"))?;

    Ok(rate)
}

#[doc = "Function to convert USD amount to KRW using the real-time exchange rate"]
/// # Arguments
/// * `usd` - Amount in USD
///
/// # Returns
/// * `Result<i64, anyhow::Error>` - Amount in KRW (rounded to nearest won)
pub async fn usd_to_krw(usd: f64) -> anyhow::Result<i64> {
    let rate: f64 = get_usd_to_krw_rate().await?;
    Ok((usd * rate).round() as i64)
}

#[doc = "Function to convert KRW amount to USD using the real-time exchange rate"]
/// # Arguments
/// * `krw` - Amount in KRW
///
/// # Returns
/// * `Result<f64, anyhow::Error>` - Amount in USD
pub async fn krw_to_usd(krw: i64) -> anyhow::Result<f64> {
    let rate: f64 = get_usd_to_krw_rate().await?;
    Ok((krw as f64 / rate * 100.0).round() / 100.0)
}
