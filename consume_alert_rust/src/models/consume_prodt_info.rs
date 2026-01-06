use crate::common::*;

use crate::entity::consume_prodt_detail;
use crate::entity::consume_prodt_detail::ActiveModel;

use crate::utils_modules::time_utils::*;

#[doc = "Structure containing consumption information."]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct ConsumeProdtInfo {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub cur_timestamp: String,
    pub prodt_name: String,
    pub prodt_money: i64,
    pub prodt_type: String,
}

impl ConsumeProdtInfo {
    #[doc = "Function to convert ConsumeProductInfo to MySQL ActiveModel."]
    pub fn convert_consume_info_to_active_model(
        &self,
    ) -> anyhow::Result<consume_prodt_detail::ActiveModel> {
        /* MySQL does not store timezones, so convert to NaiveDateTime. */
        let timestamp: NaiveDateTime = get_naive_datetime_from_str(self.timestamp(), "%Y-%m-%dT%H:%M:%SZ")
            .map_err(|e| anyhow!(
                    "[ConsumeProdtInfo::convert_consume_info_to_active_model] timestamp_local: Failed to parse timestamp: {:?}",
                    e
                ))?;

        let cur_timestamp: NaiveDateTime = get_naive_datetime_from_str(self.cur_timestamp(), "%Y-%m-%dT%H:%M:%SZ")
            .map_err(|e| anyhow!(
                "[ConsumeProdtInfo::convert_consume_info_to_active_model] cur_timestamp_local: Failed to parse timestamp: {:?}",
                e
            ))?;

        let now: NaiveDateTime = Local::now().naive_utc();

        Ok(ActiveModel {
            timestamp: Set(timestamp),
            cur_timestamp: Set(cur_timestamp),
            prodt_name: Set(self.prodt_name().clone()),
            prodt_money: Set(*self.prodt_money() as i32),
            reg_dt: Set(now),
            chg_dt: Set(None),
            reg_id: Set("system".to_string()),
            chg_id: Set(None),
        })
    }
}
