use crate::common::*;


#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct PermonDatetime {
    pub date_start: NaiveDate, 
    pub date_end: NaiveDate, 
    pub n_date_start: NaiveDate, 
    pub n_date_end: NaiveDate
}