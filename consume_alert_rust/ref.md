fn push_asset(
    map: &mut HashMap<String, Vec<AssetResp>>,
    totals: &mut AssetTotals,
    asset_type: &str,
    name: String,
    amount: Decimal,
    is_krw: bool,
    rates: ExchangeRates,
) {
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