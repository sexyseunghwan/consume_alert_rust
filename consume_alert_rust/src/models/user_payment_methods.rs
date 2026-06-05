#![allow(clippy::too_many_arguments)]
use crate::common::*;
use crate::entity::user_payment_methods;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct UserPaymentMethods {
    pub payment_method_id: i64,
    pub payment_type_cd: String,
    pub payment_category_cd: String,
    pub card_id: String,
    pub card_alias: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub is_default: bool,
    pub user_seq: i64,
    pub card_company_nm: Option<String>,
}

impl From<user_payment_methods::Model> for UserPaymentMethods {
    fn from(model: user_payment_methods::Model) -> Self {
        UserPaymentMethods::new(
            model.payment_method_id,
            model.payment_type_cd,
            model.payment_category_cd,
            model.card_id,
            model.card_alias,
            model.is_active,
            DateTime::from_naive_utc_and_offset(model.created_at, Utc),
            model
                .updated_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            model.created_by,
            model.updated_by,
            model.is_default,
            model.user_seq,
            model.card_company_nm,
        )
    }
}
