use crate::common::*;

use crate::views::spent_detail_view::*;

use crate::entity::spent_detail;
use crate::entity::spent_detail::ActiveModel;

use crate::models::{consume_index_prodt_type::*, spent_detail_by_produce::*};

use crate::enums::indexing_type::*;

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct SpentDetail {
    pub spent_name: String,
    pub spent_money: i32,
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

    /// Converts this `SpentDetail` into a `SpentDetailByProduce` payload for Kafka message production.
    ///
    /// # Arguments
    ///
    /// * `spent_idx` - The database-assigned primary key of the inserted record
    /// * `consume_keyword_type` - The human-readable consumption category name
    /// * `room_seq` - The Telegram room sequence number
    /// * `indexing_type` - Whether this is an insert or delete operation
    /// * `user_id` - The user ID string
    ///
    /// # Returns
    ///
    /// Returns a `SpentDetailByProduce` instance ready for Kafka production.
    #[allow(dead_code)]
    pub fn convert_to_spent_detail_by_produce(
        &self,
        spent_idx: i64,
        consume_keyword_type: &str,
        room_seq: i64,
        indexing_type: IndexingType,
        user_id: &str,
    ) -> SpentDetailByProduce {
        let now: DateTime<Utc> = Utc::now();

        SpentDetailByProduce::new(
            spent_idx,
            self.spent_name.clone(),
            self.spent_money,
            self.spent_at.with_timezone(&Utc),
            now,
            self.user_seq,
            self.consume_keyword_type_id,
            consume_keyword_type.to_string(),
            room_seq,
            indexing_type,
            now,
            user_id.to_string(),
        )
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
