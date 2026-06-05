use crate::common::*;

use crate::entity::{earned_detail, spent_detail};
use crate::models::earned_detail::*;
use crate::models::spent_detail::*;
use crate::repository::mysql_repository::*;

use super::MysqlQueryServiceImpl;

impl<R: MysqlRepository + Send + Sync> MysqlQueryServiceImpl<R> {
    pub async fn input_earned_detail_with_transaction(
        &self,
        earned_detail_model: &EarnedDetail,
    ) -> anyhow::Result<i64> {
        let active_model: earned_detail::ActiveModel =
            earned_detail_model.to_active_model().inspect_err(|e| {
                error!(
                    "[input_earned_detail_with_transaction] Failed to convert to ActiveModel: {:#}",
                    e
                )
            })?;

        self.db_conn
            .input_earned_detail_with_transaction(active_model)
            .await
    }

    pub async fn input_prodt_detail_with_transaction(
        &self,
        spent_detail: &SpentDetail,
    ) -> anyhow::Result<i64> {
        let active_model: spent_detail::ActiveModel =
            spent_detail.to_active_model().inspect_err(|e| {
                error!(
                    "[input_prodt_detail_with_transaction] Failed to convert to ActiveModel: {:#}",
                    e
                )
            })?;

        self.db_conn
            .input_spent_detail_with_transaction(active_model)
            .await
    }

    pub async fn input_prodt_details_with_transaction(
        &self,
        spent_details: &[SpentDetail],
    ) -> anyhow::Result<Vec<i64>> {
        let mut active_models: Vec<spent_detail::ActiveModel> =
            Vec::with_capacity(spent_details.len());

        for (position, detail) in spent_details.iter().enumerate() {
            let active_model: spent_detail::ActiveModel =
                detail.to_active_model().map_err(|e| {
                    anyhow!(
                        "[MysqlQueryServiceImpl::input_prodt_details_with_transaction] \
                     Failed to convert SpentDetail at position {} to ActiveModel: {:?}",
                        position,
                        e
                    )
                })?;

            active_models.push(active_model);
        }

        self.db_conn
            .input_spent_details_with_transaction(active_models)
            .await
    }
}
