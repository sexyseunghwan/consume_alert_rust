use crate::common::*;

use crate::views::spent_detail_view::*;

use crate::entity::spent_detail;
use crate::entity::spent_detail::ActiveModel;

use crate::models::common_consume_keyword_type::*;

#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct SpentDetail {
    pub spent_name: String,
    pub spent_money: i64,
    pub spent_at: DateTime<Local>,
    pub should_index: i8,
    pub user_seq: i64,
    pub spent_group_id: i64,
    pub consume_keyword_id: i64,
}

impl SpentDetail {

    pub fn convert_spent_detail_to_active_model(&self) -> anyhow::Result<spent_detail::ActiveModel> {
        let spent_at_naive: NaiveDateTime = self.spent_at.naive_utc();
        let now: NaiveDateTime = Utc::now().naive_utc();
        
        Ok(ActiveModel {
            spent_idx: NotSet,
            spent_name: Set(self.spent_name.clone()),
            spent_money: Set(self.spent_money as i32),
            spent_at: Set(spent_at_naive),
            should_index: Set(self.should_index),
            created_at: Set(now),
            updated_at: Set(None),
            created_by: Set("system".to_string()),
            updated_by: Set(None),
            user_seq: Set(self.user_seq),
            spent_group_id: Set(self.spent_group_id),
            consume_keyword_id: Set(self.consume_keyword_id),
        })
    }


    pub fn convert_spent_detail_to_view(&self, consume_keyword_type: &CommonConsumeKeywordType) -> anyhow::Result<SpentDetailView> { 
        
        Ok(SpentDetailView {
            spent_name: self.spent_name().to_string(),
            spent_money: self.spent_money.to_formatted_string(&Locale::en),
            spent_at: self.spent_at,
            consume_keyword_type_nm: consume_keyword_type.consume_keyword_type().to_string()
        })

    }
}
