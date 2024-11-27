use crate::common::*;

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct MealCheckIndex {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub alarminfo: i64,
    pub laststamp: i64
}