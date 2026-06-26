use crate::common::*;

use crate::models::asset_resp::*;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, new)]
#[getset(get = "pub")]
pub struct Assets {
    pub total_asset_amount_krw: Decimal,
    pub asset_map: HashMap<String, Vec<AssetResp>>
}