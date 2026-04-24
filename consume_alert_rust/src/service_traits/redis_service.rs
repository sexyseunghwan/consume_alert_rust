use crate::common::*;

/// Redis service trait defining business logic operations
#[async_trait]
pub trait RedisService {
    /// Set a simple string value (non-JSON)
    async fn input_string(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u64>,
    ) -> anyhow::Result<()>;

    /// Get a simple string value (non-JSON)
    async fn find_string(&self, key: &str) -> anyhow::Result<Option<String>>;
}
