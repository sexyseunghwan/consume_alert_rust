use rust_decimal::Decimal;

use crate::common::*;
use crate::entity::currency_exchange_rate_snapshot;

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct CurrencyExchangeRateSnapshot {
    pub exchange_rate_snapshot_seq: i64,
    pub base_currency_code: String,
    pub target_currency_code: String,
    pub base_amount: Decimal,
    pub exchange_rate: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub updated_by: Option<String>,
}

impl From<currency_exchange_rate_snapshot::Model> for CurrencyExchangeRateSnapshot {
    fn from(model: currency_exchange_rate_snapshot::Model) -> Self {
        CurrencyExchangeRateSnapshot::new(
            model.exchange_rate_snapshot_seq,
            model.base_currency_code,
            model.target_currency_code,
            model.base_amount,
            model.exchange_rate,
            model.is_active,
            DateTime::from_naive_utc_and_offset(model.created_at, Utc),
            model
                .updated_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            model.created_by,
            model.updated_by,
        )
    }
}
