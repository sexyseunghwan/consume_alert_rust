use rust_decimal::Decimal;

use crate::common::*;
use crate::entity::cash_asset;

#[allow(dead_code, clippy::too_many_arguments)]
#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct CashAsset {
    pub cash_seq: i64,
    pub cash_name: String,
    pub cash: Decimal,
    pub user_seq: i64,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub updated_by: Option<String>,
}

impl From<cash_asset::Model> for CashAsset {
    fn from(model: cash_asset::Model) -> Self {
        CashAsset::new(
            model.cash_seq,
            model.cash_name,
            model.cash,
            model.user_seq,
            model.currency_code,
            DateTime::from_naive_utc_and_offset(model.created_at, Utc),
            model
                .updated_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            model.created_by,
            model.updated_by,
        )
    }
}
