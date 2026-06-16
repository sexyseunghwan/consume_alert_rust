use crate::common::*;

use crate::models::asset_resp::*;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, new)]
#[getset(get = "pub")]
pub struct Assets {
    pub total_asset_amount_krw: Decimal,
    pub asset_map: HashMap<String, Vec<AssetResp>>,
    // pub deposit_asset_list: Vec<DepositAsset>,
    // pub saving_asset_list: Vec<SavingAsset>,
    // pub stock_resp_list: Vec<StockResp>,
    // pub crypto_resp_list: Vec<CryptoResp>,
    // pub cash_asset_list: Vec<CashAsset>,
}

// impl Assets {
//     pub fn new(
//         total_asset_amount_krw: Decimal,
//         asset_resps: HashMap<String, AssetResp>,
//         deposit_asset_list: Vec<DepositAsset>,
//         saving_asset_list: Vec<SavingAsset>,
//         stock_resp_list: Vec<StockResp>,
//         crypto_resp_list: Vec<CryptoResp>,
//         cash_asset_list: Vec<CashAsset>,
//     ) -> Self {
//         Self {
//             total_asset_amount_krw,
//             asset_resps,
//             deposit_asset_list,
//             saving_asset_list,
//             stock_resp_list,
//             crypto_resp_list,
//             cash_asset_list,
//         }
//     }
// }
