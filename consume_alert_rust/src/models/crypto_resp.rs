use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, Getters, new)]
#[getset(get = "pub")]
pub struct CryptoResp {
    pub crypto_name: String,
    pub crypto_total_price: Decimal,
}
