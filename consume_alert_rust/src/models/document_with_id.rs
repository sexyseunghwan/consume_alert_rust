use crate::common::*;

#[doc = "Structure containing consumption information."]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct DocumentWithId<T> {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(flatten)]
    pub source: T,
}
