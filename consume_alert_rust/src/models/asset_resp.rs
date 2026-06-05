use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct AssetResp {
    pub asset_type: String,
    pub asset_name: String,
    pub asset_krw: Decimal,
    pub asset_usd: Decimal,
}
