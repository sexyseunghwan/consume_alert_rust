use crate::common::*;

#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub")]
pub struct FileInfo {
    pub file_name: String,
    pub file_bytes: Vec<u8>,
}
