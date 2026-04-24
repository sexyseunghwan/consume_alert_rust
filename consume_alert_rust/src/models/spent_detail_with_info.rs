use crate::common::*;
//use crate::models::spent_detail_by_produce::SpentDetailByProduce;
use crate::views::spent_detail_view::SpentDetailView;

#[derive(Debug, Clone, FromQueryResult)]
#[allow(dead_code)]
pub struct SpentDetailWithInfo {
    pub spent_idx: i64,
    pub spent_name: String,
    pub spent_money: i64,
    pub spent_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub user_seq: i64,
    pub consume_keyword_type_id: i64,
    pub consume_keyword_type: String,
    pub room_seq: i64,
    pub user_id: String,
}

impl SpentDetailWithInfo {
    /// Converts this `SpentDetailWithInfo` into a `SpentDetailView` for Telegram message display.
    ///
    /// # Returns
    ///
    /// Returns a `SpentDetailView` built from the record's fields.
    pub fn to_spent_detail_view(&self) -> SpentDetailView {
        SpentDetailView {
            spent_name: self.spent_name.clone(),
            spent_money: self.spent_money.to_formatted_string(&Locale::en),
            spent_at: Local.from_utc_datetime(&self.spent_at),
            consume_keyword_type_nm: self.consume_keyword_type.clone(),
        }
    }
}

// impl SpentDetailWithInfo {
//     /// Converts this `SpentDetailWithInfo` into a `SpentDetailByProduce` payload for Kafka production.
//     ///
//     /// # Arguments
//     ///
//     /// * `indexing_type` - Whether this is an insert or delete operation
//     ///
//     /// # Returns
//     ///
//     /// Returns a `SpentDetailByProduce` instance ready for Kafka production.
//     pub fn to_spent_detail_by_produce(&self, indexing_type: IndexingType) -> SpentDetailByProduce {
//         SpentDetailByProduce::new(
//             self.spent_idx,
//             self.spent_name.clone(),
//             self.spent_money,
//             DateTime::from_naive_utc_and_offset(self.spent_at, Utc),
//             DateTime::from_naive_utc_and_offset(self.created_at, Utc),
//             self.user_seq,
//             self.consume_keyword_type_id,
//             self.consume_keyword_type.clone(),
//             self.room_seq,
//             indexing_type,
//             Utc::now(),
//             self.user_id.clone(),
//         )
//     }
// }
