use crate::enums::range_operator::*;
use crate::models::per_datetime::*;

pub struct CommonProcessPythonDoubleDto {
    pub index_name: String,
    pub permon_datetime: PerDatetime,
    pub start_op: RangeOperator,
    pub end_op: RangeOperator,
    pub room_seq: Option<i64>,
    pub group_seq: Option<i64>,
    pub detail_yn: bool,
}
