use crate::common::*;

#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct PerDatetime {
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
    pub n_date_start: DateTime<Utc>,
    pub n_date_end: DateTime<Utc>,
}
