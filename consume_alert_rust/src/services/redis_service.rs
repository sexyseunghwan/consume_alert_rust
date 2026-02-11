use crate::common::*;
use crate::repository::redis_repository::*;

/// Redis service trait defining business logic operations
#[async_trait]
pub trait RedisService {
    /// Invalidate (delete) a cached value
    ///
    /// # Arguments
    /// * `key` - Cache key to invalidate
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if invalidation succeeds
    async fn invalidate_cache(&self, key: &str) -> anyhow::Result<()>;

    /// Check if a cache key exists
    ///
    /// # Arguments
    /// * `key` - Cache key to check
    ///
    /// # Returns
    /// * `Result<bool, anyhow::Error>` - True if key exists
    async fn cache_exists(&self, key: &str) -> anyhow::Result<bool>;

    /// Set a simple string value (non-JSON)
    ///
    /// # Arguments
    /// * `key` - Key to set
    /// * `value` - String value
    /// * `ttl_seconds` - Optional time-to-live in seconds
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if set succeeds
    async fn set_string(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> anyhow::Result<()>;

    /// Get a simple string value (non-JSON)
    ///
    /// # Arguments
    /// * `key` - Key to get
    ///
    /// # Returns
    /// * `Result<Option<String>, anyhow::Error>` - String value if exists
    async fn get_string(&self, key: &str) -> anyhow::Result<Option<String>>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct RedisServiceImpl<R: RedisRepository> {
    redis_conn: R,
}

impl<R: RedisRepository> RedisServiceImpl<R> {
    /// Cache a value with optional expiration (Generic helper method)
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `value` - Value to cache (will be serialized to JSON)
    /// * `ttl_seconds` - Optional time-to-live in seconds
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if cache succeeds
    pub async fn cache_value<T: Serialize + Send>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: Option<u64>,
    ) -> anyhow::Result<()> {
        let json_value: String = serde_json::to_string(value)
            .map_err(|e| anyhow!("[RedisServiceImpl::cache_value] Failed to serialize value: {:?}", e))?;

        match ttl_seconds {
            Some(ttl) => self.redis_conn.set_ex(key, &json_value, ttl).await,
            None => self.redis_conn.set(key, &json_value).await,
        }
    }

    /// Get cached value (Generic helper method)
    ///
    /// # Arguments
    /// * `key` - Cache key
    ///
    /// # Returns
    /// * `Result<Option<T>, anyhow::Error>` - Cached value if exists
    pub async fn get_cached_value<T: for<'de> Deserialize<'de>>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let value: Option<String> = self.redis_conn.get(key).await?;

        match value {
            Some(json_str) => {
                let deserialized: T = serde_json::from_str(&json_str)
                    .map_err(|e| anyhow!("[RedisServiceImpl::get_cached_value] Failed to deserialize value: {:?}", e))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl<R: RedisRepository + Send + Sync> RedisService for RedisServiceImpl<R> {
    async fn invalidate_cache(&self, key: &str) -> anyhow::Result<()> {
        self.redis_conn.del(key).await
    }

    async fn cache_exists(&self, key: &str) -> anyhow::Result<bool> {
        self.redis_conn.exists(key).await
    }

    async fn set_string(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> anyhow::Result<()> {
        match ttl_seconds {
            Some(ttl) => self.redis_conn.set_ex(key, value, ttl).await,
            None => self.redis_conn.set(key, value).await,
        }
    }

    async fn get_string(&self, key: &str) -> anyhow::Result<Option<String>> {
        self.redis_conn.get(key).await
    }
}
