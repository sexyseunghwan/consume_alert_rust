use crate::common::*;

use crate::models::spent_detail::*;

#[async_trait]
pub trait MysqlQueryService {
    async fn insert_prodt_detail_with_transaction(
        &self,
        spent_detail: &SpentDetail,
    ) -> anyhow::Result<i64>;
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
    async fn get_user_id_by_seq(&self, user_seq: i64) -> anyhow::Result<Option<String>>;
}
