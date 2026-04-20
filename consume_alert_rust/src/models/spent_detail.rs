use crate::common::*;

use crate::views::spent_detail_view::*;

use crate::entity::spent_detail;
use crate::entity::spent_detail::ActiveModel;

use crate::models::consume_index_prodt_type::*;


#[allow(clippy::too_many_arguments)]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct SpentDetail {
    pub spent_name: String,
    pub spent_money: i64,
    pub spent_at: DateTime<Local>,
    pub should_index: i8,
    pub user_seq: i64,
    pub spent_group_id: i64,
    pub consume_keyword_type_id: i64,
    pub room_seq: i64,
    pub payment_method_id: i64,
}

impl SpentDetail {
    /// Converts this `SpentDetail` domain model into a SeaORM `ActiveModel` for database insertion.
    ///
    /// # Returns
    ///
    /// Returns `Ok(spent_detail::ActiveModel)` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if any field conversion fails.
    pub fn convert_spent_detail_to_active_model(
        &self,
    ) -> anyhow::Result<spent_detail::ActiveModel> {
        let spent_at_naive: NaiveDateTime = self.spent_at.naive_utc();
        let now: NaiveDateTime = Utc::now().naive_utc();

        Ok(ActiveModel {
            spent_idx: NotSet,
            spent_name: Set(self.spent_name.clone()),
            spent_money: Set(self.spent_money),
            spent_at: Set(spent_at_naive),
            should_index: Set(self.should_index),
            created_at: Set(now),
            updated_at: Set(None),
            created_by: Set("system".to_string()),
            updated_by: Set(None),
            user_seq: Set(self.user_seq),
            spent_group_id: Set(self.spent_group_id),
            consume_keyword_type_id: Set(self.consume_keyword_type_id),
            room_seq: Set(self.room_seq),
            payment_method_id: Set(self.payment_method_id),
        })
    }

    /// Converts this `SpentDetail` into a `SpentDetailView` for Telegram message display.
    ///
    /// # Arguments
    ///
    /// * `spent_type` - The resolved consumption category information
    ///
    /// # Returns
    ///
    /// Returns `Ok(SpentDetailView)` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the view construction fails.
    pub fn convert_spent_detail_to_view(
        &self,
        spent_type: &ConsumingIndexProdtType,
    ) -> anyhow::Result<SpentDetailView> {
        Ok(SpentDetailView {
            spent_name: self.spent_name().to_string(),
            spent_money: self.spent_money.to_formatted_string(&Locale::en),
            spent_at: self.spent_at,
            consume_keyword_type_nm: spent_type.consume_keyword_type().to_string(),
        })
    }
}
