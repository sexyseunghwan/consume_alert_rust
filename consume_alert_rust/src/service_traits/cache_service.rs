use crate::common::*;

#[async_trait]
pub trait CacheService {
    async fn find_user_seq(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>>;
    async fn find_telegram_room_seq(
        &self,
        user_seq: i64,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>>;
    async fn find_telegram_group_seq(
        &self,
        user_seq: i64,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>>;
}
