//! Spent detail with KST timezone for display purposes.
//!
//! This module provides the data structure for spent detail information
//! with all datetime fields converted to Korean Standard Time (KST).

use crate::common::*;

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, Serialize, Getters, new)]
#[getset(get = "pub")]
pub struct SpentDetailByEsKst {
    /// Primary key of the spent detail
    pub spent_idx: i64,

    /// Name/description of the spending
    pub spent_name: String,

    /// Amount spent
    pub spent_money: i64,

    /// Date and time of the spending (KST)
    pub spent_at: DateTime<chrono_tz::Tz>,

    /// Record creation timestamp (KST)
    pub created_at: DateTime<chrono_tz::Tz>,

    /// User identifier
    pub user_seq: i64,

    /// Keyword type identifier
    pub consume_keyword_type_id: i64,

    /// The type/category of the keyword
    pub consume_keyword_type: String,

    /// Telegram room identifier
    pub room_seq: i64,

    /// Record indexing timestamp (KST)
    pub produced_at: Option<DateTime<chrono_tz::Tz>>,
}
