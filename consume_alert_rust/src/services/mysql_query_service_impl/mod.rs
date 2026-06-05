mod delete;
mod insert;
mod select;
mod update;

use crate::common::*;

use crate::models::{
    cash_asset::*, crypto_resp::*, currency_exchange_rate_snapshot::*, deposit_asset::*,
    earned_detail::*, saving_asset::*, spent_detail::*, spent_detail_with_info::*, stock_resp::*,
    user_payment_methods::*,
};
use crate::repository::mysql_repository::*;

use crate::service_traits::mysql_query_service::*;

#[derive(Debug, Getters, Clone, new)]
pub struct MysqlQueryServiceImpl<R: MysqlRepository> {
    pub db_conn: R,
}

#[async_trait]
impl<R: MysqlRepository + Send + Sync> MysqlQueryService for MysqlQueryServiceImpl<R> {
    async fn input_earned_detail_with_transaction(
        &self,
        earned_detail_model: &EarnedDetail,
    ) -> anyhow::Result<i64> {
        self.input_earned_detail_with_transaction(earned_detail_model)
            .await
    }

    async fn input_prodt_detail_with_transaction(
        &self,
        spent_detail: &SpentDetail,
    ) -> anyhow::Result<i64> {
        self.input_prodt_detail_with_transaction(spent_detail).await
    }

    async fn input_prodt_details_with_transaction(
        &self,
        spent_details: &[SpentDetail],
    ) -> anyhow::Result<Vec<i64>> {
        self.input_prodt_details_with_transaction(spent_details)
            .await
    }

    async fn has_telegram_room_by_token_and_id(
        &self,
        room_token: &str,
        user_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        self.has_telegram_room_by_token_and_id(room_token, user_id)
            .await
    }

    async fn find_telegram_room_seq_by_token_and_userseq(
        &self,
        room_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>> {
        self.find_telegram_room_seq_by_token_and_userseq(room_token, user_seq)
            .await
    }

    async fn find_telegram_group_seq_by_token_and_userseq(
        &self,
        room_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>> {
        self.find_telegram_group_seq_by_token_and_userseq(room_token, user_seq)
            .await
    }

    async fn find_user_id_by_seq(&self, user_seq: i64) -> anyhow::Result<Option<String>> {
        self.find_user_id_by_seq(user_seq).await
    }

    async fn find_latest_spent_idx(
        &self,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<Option<i64>> {
        self.find_latest_spent_idx(user_seq, room_seq).await
    }

    async fn find_latest_spent_detail(
        &self,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<Option<SpentDetailWithInfo>> {
        self.find_latest_spent_detail(user_seq, room_seq).await
    }

    async fn find_spent_detail_with_info(
        &self,
        spent_idx: i64,
    ) -> anyhow::Result<Option<SpentDetailWithInfo>> {
        self.find_spent_detail_with_info(spent_idx).await
    }

    async fn delete_spent_detail_with_transaction(&self, spent_idx: i64) -> anyhow::Result<()> {
        self.delete_spent_detail_with_transaction(spent_idx).await
    }

    async fn find_user_payment_methods(
        &self,
        user_seq: i64,
        is_default: bool,
    ) -> anyhow::Result<Vec<UserPaymentMethods>> {
        self.find_user_payment_methods(user_seq, is_default).await
    }

    async fn find_currency_exchange_rate_snapshot(
        &self,
        base_currency_code: &str,
        target_currency_code: &str,
    ) -> anyhow::Result<Vec<CurrencyExchangeRateSnapshot>> {
        self.find_currency_exchange_rate_snapshot(base_currency_code, target_currency_code)
            .await
    }

    async fn find_deposit_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<DepositAsset>> {
        self.find_deposit_asset(user_seq, currency_code).await
    }

    async fn find_saving_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<SavingAsset>> {
        self.find_saving_asset(user_seq, currency_code).await
    }

    async fn find_stock_response(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<StockResp>> {
        self.find_stock_response(user_seq, currency_code).await
    }

    async fn find_crypto_response(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<CryptoResp>> {
        self.find_crypto_response(user_seq, currency_code).await
    }

    async fn find_cash_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<CashAsset>> {
        self.find_cash_asset(user_seq, currency_code).await
    }
}
