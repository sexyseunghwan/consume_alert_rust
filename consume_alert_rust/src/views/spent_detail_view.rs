use crate::common::*;

#[doc = "Structure containing spent detail information."]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct SpentDetailView {
    pub spent_name: String,
    pub spent_money: String,
    pub spent_at: DateTime<Local>,
    pub consume_keyword_type_nm: String
}