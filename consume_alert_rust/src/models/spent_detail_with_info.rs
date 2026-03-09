use crate::common::*;
use crate::enums::indexing_type::IndexingType;
use crate::models::spent_detail_by_produce::SpentDetailByProduce;

#[derive(Debug, Clone, FromQueryResult)]
pub struct SpentDetailWithInfo {
    pub spent_idx: i64,
    pub spent_name: String,
    pub spent_money: i32,
    pub spent_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub user_seq: i64,
    pub consume_keyword_type_id: i64,
    pub consume_keyword_type: String,
    pub room_seq: i64,
    pub user_id: String,
}

impl SpentDetailWithInfo {
    pub fn to_spent_detail_by_produce(&self, indexing_type: IndexingType) -> SpentDetailByProduce {
        SpentDetailByProduce::new(
            self.spent_idx,
            self.spent_name.clone(),
            self.spent_money,
            DateTime::from_naive_utc_and_offset(self.spent_at, Utc),
            DateTime::from_naive_utc_and_offset(self.created_at, Utc),
            self.user_seq,
            self.consume_keyword_type_id,
            self.consume_keyword_type.clone(),
            self.room_seq,
            indexing_type,
            Utc::now(),
            self.user_id.clone(),
        )
    }
}
