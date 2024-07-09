use crate::common::*;


#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ConsumeInfo {
    pub timestamp: String,
    pub prodt_name: String,
    pub prodt_money: i32,
    pub prodt_type: String
}

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ProdtDetailInfo {
    pub keyword: String,
    pub bias_value: i32
}

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ProdtTypeInfo {
    pub keyword_type: String,
    pub prodt_detail_list: Vec<ProdtDetailInfo>
}

