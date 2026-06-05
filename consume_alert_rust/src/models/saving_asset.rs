#![allow(dead_code, clippy::too_many_arguments)]
use crate::common::*;
use crate::entity::saving_asset;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct SavingAsset {
    pub saving_seq: i64,
    pub saving_name: String,
    pub saving_amount: Decimal,
    pub accum_saving_amount: Decimal,
    pub interest_rate: Decimal,
    pub term_month: i32,
    pub saving_start_date: DateTime<Utc>,
    pub saving_end_date: DateTime<Utc>,
    pub is_terminated: bool,
    pub user_seq: i64,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub updated_by: Option<String>,
}

impl From<saving_asset::Model> for SavingAsset {
    fn from(model: saving_asset::Model) -> Self {
        SavingAsset::new(
            model.saving_seq,
            model.saving_name,
            model.saving_amount,
            model.accum_saving_amount,
            model.interest_rate,
            model.term_month,
            DateTime::from_naive_utc_and_offset(model.saving_start_date, Utc),
            DateTime::from_naive_utc_and_offset(model.saving_end_date, Utc),
            model.is_terminated,
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
