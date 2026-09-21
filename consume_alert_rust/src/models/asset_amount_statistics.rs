use crate::common::*;

use crate::models::user_asset_snapshot_summary::*;

#[derive(Debug, Clone)]
pub struct AssetAmountStatistics {
    pub min: Decimal,
    pub max: Decimal,
    pub average: Decimal,
    pub median: Decimal,
    pub cur_val: Decimal,
    pub price_range: Decimal,
    pub price_range_rate: Decimal,
    /// Highest local maximum (양쪽 이웃보다 높은 시점) in the snapshot's chronological order.
    pub resistance_line: Decimal,
    /// Lowest local minimum (양쪽 이웃보다 낮은 시점) in the snapshot's chronological order.
    pub support_line: Decimal,
}

pub fn calculate_asset_amount_statistics(
    summaries: &[UserAssetSnapshotSummary],
) -> Option<AssetAmountStatistics> {
    if summaries.is_empty() {
        return None;
    }

    let mut amounts: Vec<Decimal> = summaries
        .iter()
        .map(|summary| summary.total_asset_amount)
        .collect();

    // 가장 최근(마지막) 스냅샷 값 - 정렬 전에 구해야 시간 순서가 보존됨
    let cur_val: Decimal = *amounts.last()?;

    // 정렬 전 시계열(발생 순서) 기준으로 극대값/극소값 탐색 - 양쪽 이웃보다 크면 극대, 작으면 극소
    let mut local_maxima: Vec<Decimal> = Vec::new();
    let mut local_minima: Vec<Decimal> = Vec::new();

    // saturating_sub(1) -> 1을 빼주되 0보다 작아지지 않게하는 로직
    for i in 1..amounts.len().saturating_sub(1) {
        let prev: Decimal = amounts[i - 1];
        let cur: Decimal = amounts[i];
        let next: Decimal = amounts[i + 1];

        if cur > prev && cur > next {
            local_maxima.push(cur);
        } else if cur < prev && cur < next {
            local_minima.push(cur);
        }
    }

    // 중앙값 계산을 위해 오름차순 정렬
    amounts.sort();

    let count: usize = amounts.len();
    let min: Decimal = amounts[0];
    let max: Decimal = amounts[count - 1];

    // 자산 변동폭
    let price_range: Decimal = (max - min).abs();
    let price_range_rate: Decimal = (price_range / max) * Decimal::from(100);

    // 저항선/지지선 - 극대값/극소값이 없으면(구간이 짧아 이웃 비교가 불가능하면) 전체 최댓값/최솟값으로 대체

    // 저항선 = 극대값들 중 최댓값
    let resistance_line: Decimal = local_maxima.iter().copied().max().unwrap_or(max);

    // 지지선 = 극소값들 중 최솟값
    let support_line: Decimal = local_minima.iter().copied().min().unwrap_or(min);


    let sum: Decimal = amounts.iter().copied().sum();
    let average: Decimal = sum / Decimal::from(count as u64);

    let median: Decimal = if count % 2 == 1 {
        // 홀수: 가운데 값
        amounts[count / 2]
    } else {
        // 짝수: 가운데 두 값의 평균
        let left: Decimal = amounts[count / 2 - 1];
        let right: Decimal = amounts[count / 2];

        (left + right) / Decimal::from(2u32)
    };

    Some(AssetAmountStatistics {
        min,
        max,
        average,
        median,
        cur_val,
        price_range,
        price_range_rate,
        resistance_line,
        support_line,
    })
}