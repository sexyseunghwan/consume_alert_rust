use crate::common::*;

use crate::models::asset_resp::*;
use crate::models::stock_resp::StockResp;
use crate::utils_modules::numeric_utils::format_decimal_with_commas;

pub const SEP: &str = "--------------------------------------------";

#[derive(Clone, Copy)]
pub struct ExchangeRates {
    pub usd_to_krw: Decimal,
    pub krw_to_usd: Decimal,
}

pub struct AssetTotals {
    pub krw: Decimal,
    pub usd: Decimal,
}

/// All raw asset data gathered for a user before building the summary/pie/stock messages.
pub struct AssetCollection {
    pub asset_map: HashMap<String, Vec<AssetResp>>,
    pub totals: AssetTotals,
    pub stock_list: Vec<StockResp>,
    pub total_stock_amount_krw: Decimal,
}

/// Records one asset amount into `map`/`totals`, converting it to both KRW and USD via `rates`.
pub fn push_asset(
    map: &mut HashMap<String, Vec<AssetResp>>,
    totals: &mut AssetTotals,
    asset_type: &str,
    name: String,
    amount: Decimal,
    is_krw: bool,
    rates: ExchangeRates,
) {
    /* 해당 자산의 달러/원화 가치 모두 저장하고 있음. */
    let (krw, usd) = if is_krw {
        totals.krw += amount;
        (amount, amount * rates.krw_to_usd)
    } else {
        totals.usd += amount;
        (amount * rates.usd_to_krw, amount)
    };
    map.entry(asset_type.to_string())
        .or_default()
        .push(AssetResp::new(asset_type.to_string(), name, krw, usd));
}

/// Builds the Telegram summary message for every asset section (deposit/saving/stock/crypto/cash).
pub fn build_asset_message(
    asset_map: &HashMap<String, Vec<AssetResp>>,
    totals: &AssetTotals,
    rates: ExchangeRates,
) -> String {
    let grand_krw: Decimal = totals.krw + (totals.usd * rates.usd_to_krw);
    let grand_usd: Decimal = totals.usd + (totals.krw * rates.krw_to_usd);

    let sections: &[(&str, &str)] = &[
        ("Deposit", "예금성 자산"),
        ("Saving", "적금성 자산"),
        ("Stock", "주식성 자산"),
        ("Crypto", "크립토성 자산"),
        ("Cash", "현금성 자산"),
    ];
    let mut msg: String = format!(
        "총자산 = {}₩ // {:.2}$\n",
        format_decimal_with_commas(grand_krw, 0, false),
        grand_usd.round_dp(2),
    );

    for (key, label) in sections {
        msg.push_str(&format!("{}\n[{}]\n", SEP, label));
        let assets: &[AssetResp] = asset_map.get(*key).map(Vec::as_slice).unwrap_or(&[]);

        let mut section_krw: Decimal = Decimal::ZERO;
        let mut section_usd: Decimal = Decimal::ZERO;

        if assets.is_empty() {
            msg.push_str("  (없음)\n");
        } else {
            for asset in assets {
                msg.push_str(&format!(
                    "*  {} : {}₩ ({:.2}$)\n",
                    asset.asset_name(),
                    format_decimal_with_commas(asset.asset_krw, 0, false),
                    asset.asset_usd.round_dp(2),
                ));
                section_krw += asset.asset_krw;
                section_usd += asset.asset_usd;
            }
        }

        msg.push_str(&format!(
            "{} 총계 : {}₩ ({:.2}$)\n",
            label,
            format_decimal_with_commas(section_krw, 0, false),
            section_usd.round_dp(2),
        ));
    }

    msg.push_str(SEP);
    msg
}
