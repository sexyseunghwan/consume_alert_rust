use crate::common::*;
use crate::repository::redis_repository::*;

use crate::service_traits::redis_service::*;

#[derive(Debug, Getters, Clone, new)]
pub struct RedisServiceImpl<R: RedisRepository> {
    redis_conn: R,
}

#[async_trait]
impl<R: RedisRepository + Send + Sync> RedisService for RedisServiceImpl<R> {
    async fn set_string(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u64>,
    ) -> anyhow::Result<()> {
        match ttl_seconds {
            Some(ttl) => self.redis_conn.set_ex(key, value, ttl).await,
            None => self.redis_conn.set(key, value).await,
        }
    }

    async fn get_string(&self, key: &str) -> anyhow::Result<Option<String>> {
        self.redis_conn.get(key).await
    }
}
