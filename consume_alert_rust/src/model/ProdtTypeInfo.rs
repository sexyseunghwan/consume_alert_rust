use crate::common::*;
use crate::model::ProdtDetailInfo::*;

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ProdtTypeInfo {
    pub keyword_type: String,
    pub prodt_detail_list: Vec<ProdtDetailInfo>,
}
