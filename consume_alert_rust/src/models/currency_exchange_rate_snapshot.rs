use rust_decimal::Decimal;

use crate::common::*;
use crate::entity::currency_exchange_rate_snapshot;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters)]
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
        CurrencyExchangeRateSnapshot {
            exchange_rate_snapshot_seq: model.exchange_rate_snapshot_seq,
            base_currency_code: model.base_currency_code,
            target_currency_code: model.target_currency_code,
            base_amount: model.base_amount,
            exchange_rate: model.exchange_rate,
            is_active: model.is_active,
            created_at: DateTime::from_naive_utc_and_offset(model.created_at, Utc),
            updated_at: model
                .updated_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}
