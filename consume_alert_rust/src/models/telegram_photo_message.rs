use crate::common::*;

use crate::models::{asset_amount_statistics::*, user_asset_snapshot_summary::*};
use crate::utils_modules::numeric_utils::format_decimal_with_commas;

#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct TelegramPhotoMessage {
    pub photo_bytes: Vec<u8>,
    pub message: String,
}

pub fn asset_history_generator(
    label: &str,
    photo_bytes: Vec<u8>,
    start_utc: DateTime<Utc>,
    end_utc: DateTime<Utc>,
    snapshots: &[UserAssetSnapshotSummary],
) -> anyhow::Result<TelegramPhotoMessage> {
    if snapshots.is_empty() {
        return Err(anyhow!(
            "[TelegramPhotoMessage::asset_history_generator] The `snapshots` data is empty."
        ));
    }
    
    let start_kst: DateTime<chrono_tz::Tz> = start_utc.with_timezone(&Seoul);
    let end_kst: DateTime<chrono_tz::Tz> = end_utc.with_timezone(&Seoul);

    let asset_amount_statistics: AssetAmountStatistics =
        calculate_asset_amount_statistics(snapshots)
            .ok_or_else(|| {
                anyhow!(
                    "[TelegramPhotoMessage::asset_history_generator] Statistics caculation failed."
                )
            })
            .inspect_err(|e| error!("{:#}", e))?;

    let message: String = format!(
        "[{} 자산 변동 통계]\n기간 : {} ~ {}\n최소값 : {}₩\n최대값 : {}₩\n평균값 : {}₩\n중앙값 : {}₩\n현재값 : {}₩\n변동폭 : {}₩ ({:.2}%)\n저항선 : {}₩\n지지선 : {}₩",
        label,
        start_kst.format("%Y-%m-%d"),
        end_kst.format("%Y-%m-%d"),
        format_decimal_with_commas(asset_amount_statistics.min, 0, false),
        format_decimal_with_commas(asset_amount_statistics.max, 0, false),
        format_decimal_with_commas(asset_amount_statistics.average, 0, false),
        format_decimal_with_commas(asset_amount_statistics.median, 0, false),
        format_decimal_with_commas(asset_amount_statistics.cur_val, 0, false),
        format_decimal_with_commas(asset_amount_statistics.price_range, 0, false),
        asset_amount_statistics.price_range_rate,
        format_decimal_with_commas(asset_amount_statistics.resistance_line, 0, false),
        format_decimal_with_commas(asset_amount_statistics.support_line, 0, false),
    );

    Ok(TelegramPhotoMessage {
        photo_bytes,
        message,
    })
}