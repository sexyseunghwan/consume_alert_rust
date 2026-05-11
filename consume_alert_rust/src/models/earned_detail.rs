use crate::common::*;

use rust_decimal::Decimal;

use crate::entity::earned_detail;
use crate::entity::earned_detail::ActiveModel;

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct EarnedDetail {
    pub earned_name: String,
    pub earned_money: i64,
    pub earned_money_dollor: Decimal,
    pub earned_at: DateTime<Local>,
    pub user_seq: i64,
}

impl EarnedDetail {
    pub fn to_active_model(&self) -> anyhow::Result<earned_detail::ActiveModel> {
        let earned_at_naive: NaiveDateTime = self.earned_at.naive_utc();
        let now: NaiveDateTime = Utc::now().naive_utc();

        Ok(ActiveModel {
            earned_idx: NotSet,
            earned_name: Set(self.earned_name.clone()),
            earned_money: Set(self.earned_money),
            earned_money_dollor: Set(self.earned_money_dollor),
            earned_at: Set(earned_at_naive),
            created_at: Set(now),
            updated_at: Set(None),
            created_by: Set("system".to_string()),
            updated_by: Set(None),
            user_seq: Set(self.user_seq),
        })
    }
}
