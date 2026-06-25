use crate::common::*;

use crate::models::{
    cash_asset::*, crypto_resp::*, currency_exchange_rate_snapshot::*, deposit_asset::*,
    earned_detail::*, saving_asset::*, spent_detail::*, spent_detail_with_info::*, stock_resp::*,
    user_payment_methods::*,
};

#[async_trait]
pub trait MysqlQueryService {
    async fn input_earned_detail_with_transaction(
        &self,
        earned_detail: &EarnedDetail,
    ) -> anyhow::Result<i64>;
    async fn input_prodt_detail_with_transaction(
        &self,
        spent_detail: &SpentDetail,
    ) -> anyhow::Result<i64>;
    #[allow(dead_code)]
    async fn input_prodt_details_with_transaction(
        &self,
        spent_details: &[SpentDetail],
    ) -> anyhow::Result<Vec<i64>>;
    async fn has_telegram_room_by_token_and_id(
        &self,
        room_token: &str,
        user_id: &str,
    ) -> anyhow::Result<Option<i64>>;
    async fn find_telegram_room_seq_by_token_and_userseq(
        &self,
        room_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>>;
    async fn find_telegram_group_seq_by_token_and_userseq(
        &self,
        room_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>>;
    #[allow(dead_code)]
    async fn find_user_id_by_seq(&self, user_seq: i64) -> anyhow::Result<Option<String>>;
    #[allow(dead_code)]
    async fn find_latest_spent_idx(
        &self,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<Option<i64>>;
    async fn find_latest_spent_detail(
        &self,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<Option<SpentDetailWithInfo>>;
    #[allow(dead_code)]
    async fn find_spent_detail_with_info(
        &self,
        spent_idx: i64,
    ) -> anyhow::Result<Option<SpentDetailWithInfo>>;
    async fn delete_spent_detail_with_transaction(&self, spent_idx: i64) -> anyhow::Result<()>;
    async fn find_user_payment_methods(
        &self,
        user_seq: i64,
        is_default: bool,
    ) -> anyhow::Result<Vec<UserPaymentMethods>>;
    async fn find_currency_exchange_rate_snapshot(
        &self,
        base_currency_code: &str,
        target_currency_code: &str,
    ) -> anyhow::Result<Vec<CurrencyExchangeRateSnapshot>>;

    async fn find_deposit_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<DepositAsset>>;

    async fn find_saving_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<SavingAsset>>;
    /*
        SELECT
            s.stock_seq,
            s.stock_name,
            s.stock_price,
            sa.stock_cnt,
            sa.avg_purchase_price
        FROM STOCK_ASSET sa
        INNER JOIN STOCK s ON sa.stock_seq = s.stock_seq
        INNER JOIN STOCK_TYPE st ON s.market_seq = st.market_seq
        WHERE st.currency_code = 'USD'
        AND sa.user_seq = 1
        AND sa.stock_cnt > 0;
    */
    async fn find_stock_response(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<StockResp>>;

    async fn find_crypto_response(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<CryptoResp>>;

    async fn find_cash_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<CashAsset>>;
}
