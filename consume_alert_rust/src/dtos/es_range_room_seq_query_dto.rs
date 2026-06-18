use crate::common::*;
use crate::enums::range_operator::*;

pub struct EsRangeRoomSeqQueryDto {
    pub index_name: String,
    pub range_field: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub start_op: RangeOperator,
    pub end_op: RangeOperator,
    pub order_by_field: String,
    pub asc_yn: bool,
    pub aggs_field: String,
    pub room_seq: i64,
}
