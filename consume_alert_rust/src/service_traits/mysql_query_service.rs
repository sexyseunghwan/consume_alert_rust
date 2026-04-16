use crate::common::*;

use crate::models::spent_detail::*;
use crate::models::spent_detail_with_info::*;
use crate::models::user_payment_methods::*;

#[async_trait]
pub trait MysqlQueryService {
    async fn insert_prodt_detail_with_transaction(
        &self,
        spent_detail: &SpentDetail,
    ) -> anyhow::Result<i64>;
    #[allow(dead_code)]
    async fn insert_prodt_details_with_transaction(
        &self,
        spent_details: &[SpentDetail],
    ) -> anyhow::Result<Vec<i64>>;
    async fn exists_telegram_room_by_token_and_id(
        &self,
        room_token: &str,
        user_id: &str,
    ) -> anyhow::Result<Option<i64>>;
    async fn get_telegram_room_seq_by_token_and_userseq(
        &self,
        room_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>>;
    #[allow(dead_code)]
    async fn get_user_id_by_seq(&self, user_seq: i64) -> anyhow::Result<Option<String>>;
    async fn get_latest_spent_idx(
        &self,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<Option<i64>>;
    async fn get_latest_spent_detail(
        &self,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<Option<SpentDetailWithInfo>>;
    async fn get_spent_detail_with_info(
        &self,
        spent_idx: i64,
    ) -> anyhow::Result<Option<SpentDetailWithInfo>>;
    async fn delete_spent_detail_with_transaction(&self, spent_idx: i64) -> anyhow::Result<()>;
    async fn get_user_payment_methods(
        &self,
        user_seq: i64,
        is_default: bool,
    ) -> anyhow::Result<Vec<UserPaymentMethods>>;
}
