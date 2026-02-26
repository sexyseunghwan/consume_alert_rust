//! Spent detail with relations model for Elasticsearch indexing.
//!
//! This module provides the data structure for indexing spent detail information
//! along with related keyword, keyword type, and telegram room data.

use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct SpentDetailByProduce {
    /// Primary key of the spent detail
    pub spent_idx: i64,

    /// Name/description of the spending
    pub spent_name: String,

    /// Amount spent
    pub spent_money: i32,

    /// Date and time of the spending
    pub spent_at: DateTime<Utc>,

    /// Record creation timestamp
    pub created_at: DateTime<Utc>,

    /// User identifier
    pub user_seq: i64,

    /// Keyword type identifier
    pub consume_keyword_type_id: i64,

    /// The type/category of the keyword
    pub consume_keyword_type: String,

    /// Telegram room identifier
    pub room_seq: i64,

    /// Record indexing timestamp
    pub produced_at: DateTime<Utc>,
}
