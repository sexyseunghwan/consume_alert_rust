use crate::common::*;

use crate::entity::common_consume_keyword_type;
use crate::entity::common_consume_keyword_type::ActiveModel;

#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct CommonConsumeKeywordType {
    pub consume_keyword_type_id: i64,
    pub consume_keyword_type: String,
}

impl CommonConsumeKeywordType {
    #[doc = "Convert CommonConsumeKeywordType to SeaORM ActiveModel"]
    /// # Returns
    /// * `anyhow::Result<ActiveModel>` - ActiveModel for database insertion
    pub fn convert_to_active_model(&self) -> anyhow::Result<common_consume_keyword_type::ActiveModel> {
        let now: NaiveDateTime = Utc::now().naive_utc();

        Ok(ActiveModel {
            consume_keyword_type_id: Set(self.consume_keyword_type_id),
            consume_keyword_type: Set(self.consume_keyword_type.clone()),
            created_at: Set(now),
            updated_at: Set(None),
            created_by: Set("system".to_string()),
            updated_by: Set(None),
        })
    }
}
