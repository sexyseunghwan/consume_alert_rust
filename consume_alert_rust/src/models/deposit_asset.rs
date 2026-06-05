#![allow(dead_code, clippy::too_many_arguments)]
use crate::common::*;
use crate::entity::deposit_asset;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct DepositAsset {
    pub deposit_seq: i64,
    pub deposit_name: String,
    pub deposit_amount: Decimal,
    pub interest_rate: Decimal,
    pub deposit_start_date: DateTime<Utc>,
    pub deposit_end_date: DateTime<Utc>,
    pub user_seq: i64,
    pub currency_code: String,
    pub is_terminated: bool,
    pub term_month: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub updated_by: Option<String>,
}

impl From<deposit_asset::Model> for DepositAsset {
    fn from(model: deposit_asset::Model) -> Self {
        DepositAsset::new(
            model.deposit_seq,
            model.deposit_name,
            model.deposit_amount,
            model.interest_rate,
            DateTime::from_naive_utc_and_offset(model.deposit_start_date, Utc),
            DateTime::from_naive_utc_and_offset(model.deposit_end_date, Utc),
            model.user_seq,
            model.currency_code,
            model.is_terminated,
            model.term_month,
            DateTime::from_naive_utc_and_offset(model.created_at, Utc),
            model
                .updated_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            model.created_by,
            model.updated_by,
        )
    }
}
