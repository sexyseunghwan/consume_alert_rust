use crate::common::*;

/// Enum to represent different types of Redis connections
pub enum RedisConnectionType {
    Single(MultiplexedConnection),
    Cluster(ClusterConnection),
}

/// Redis repository trait defining common Redis operations
#[async_trait]
pub trait RedisRepository {
    /// Get a value by key
    ///
    /// # Arguments
    /// * `key` - The key to retrieve
    ///
    /// # Returns
    /// * `Result<Option<String>, anyhow::Error>` - The value if exists, None otherwise
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>>;

    /// Set a key-value pair
    ///
    /// # Arguments
    /// * `key` - The key to set
    /// * `value` - The value to set
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if set succeeds
    async fn set(&self, key: &str, value: &str) -> anyhow::Result<()>;

    /// Set a key-value pair with expiration time
    ///
    /// # Arguments
    /// * `key` - The key to set
    /// * `value` - The value to set
    /// * `seconds` - Expiration time in seconds
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if set succeeds
    async fn set_ex(&self, key: &str, value: &str, seconds: u64) -> anyhow::Result<()>;

    /// Delete a key
    ///
    /// # Arguments
    /// * `key` - The key to delete
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if delete succeeds
    async fn del(&self, key: &str) -> anyhow::Result<()>;

    /// Check if a key exists
    ///
    /// # Arguments
    /// * `key` - The key to check
    ///
    /// # Returns
    /// * `Result<bool, anyhow::Error>` - True if key exists, false otherwise
    async fn exists(&self, key: &str) -> anyhow::Result<bool>;

    /// Set expiration time for a key
    ///
    /// # Arguments
    /// * `key` - The key to set expiration
    /// * `seconds` - Expiration time in seconds
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if expire succeeds
    async fn expire(&self, key: &str, seconds: u64) -> anyhow::Result<()>;
}

/// Redis repository implementation
pub struct RedisRepositoryImpl {
    conn: RedisConnectionType,
}

impl RedisRepositoryImpl {
    /// Create a new Redis repository instance
    /// Supports both single node and cluster modes based on REDIS_URL format
    ///
    /// # Environment Variables
    /// * `REDIS_URL` - Redis connection URL
    ///   - Single node: "redis://127.0.0.1:6379"
    ///   - Cluster: "redis://node1:6379,redis://node2:6379,redis://node3:6379"
    ///
    /// # Returns
    /// * `Result<Self, anyhow::Error>` - New instance or error
    pub async fn new() -> anyhow::Result<Self> {
        let redis_url: String = env::var("REDIS_URL")
            .context("[RedisRepositoryImpl::new] ‘REDIS_URL’ cannot be found.")?;

        // Check if the URL contains multiple nodes (cluster mode)
        let conn: RedisConnectionType = if redis_url.contains(',') {
            // Cluster mode
            let nodes: Vec<&str> = redis_url.split(',').collect();
            let node_count: usize = nodes.len();
            let cluster_client: ClusterClient = ClusterClient::new(nodes).map_err(|e| {
                anyhow!(
                    "[RedisRepositoryImpl::new] Failed to create Redis cluster client: {:?}",
                    e
                )
            })?;

            let cluster_conn: ClusterConnection =
                cluster_client.get_async_connection().await.map_err(|e| {
                    anyhow!(
                        "[RedisRepositoryImpl::new] Failed to connect to Redis cluster: {:?}",
                        e
                    )
                })?;

            info!(
                "[RedisRepositoryImpl::new] Connected to Redis cluster with {} nodes",
                node_count
            );
            RedisConnectionType::Cluster(cluster_conn)
        } else {
            // Single node mode
            let client = redisClient::open(redis_url.as_str()).map_err(|e| {
                anyhow!(
                    "[RedisRepositoryImpl::new] Failed to create Redis client: {:?}",
                    e
                )
            })?;

            let single_conn: MultiplexedConnection = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| {
                    anyhow!(
                        "[RedisRepositoryImpl::new] Failed to connect to Redis: {:?}",
                        e
                    )
                })?;

            info!("[RedisRepositoryImpl::new] Connected to Redis single node");
            RedisConnectionType::Single(single_conn)
        };

        Ok(Self { conn })
    }
}

#[async_trait]
impl RedisRepository for RedisRepositoryImpl {
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        match &self.conn {
            RedisConnectionType::Single(conn) => {
                let mut conn = conn.clone();
                let result: Option<String> = conn.get(key).await.map_err(|e: RedisError| {
                    anyhow!(
                        "[RedisRepositoryImpl::get] Failed to get key '{}': {:?}",
                        key,
                        e
                    )
                })?;
                Ok(result)
            }
            RedisConnectionType::Cluster(conn) => {
                let mut conn = conn.clone();
                let result: Option<String> = conn.get(key).await.map_err(|e: RedisError| {
                    anyhow!(
                        "[RedisRepositoryImpl::get] Failed to get key '{}': {:?}",
                        key,
                        e
                    )
                })?;
                Ok(result)
            }
        }
    }

    async fn set(&self, key: &str, value: &str) -> anyhow::Result<()> {
        match &self.conn {
            RedisConnectionType::Single(conn) => {
                let mut conn = conn.clone();
                conn.set(key, value).await.map_err(|e: RedisError| {
                    anyhow!(
                        "[RedisRepositoryImpl::set] Failed to set key '{}': {:?}",
                        key,
                        e
                    )
                })?;
                Ok(())
            }
            RedisConnectionType::Cluster(conn) => {
                let mut conn = conn.clone();
                conn.set(key, value).await.map_err(|e: RedisError| {
                    anyhow!(
                        "[RedisRepositoryImpl::set] Failed to set key '{}': {:?}",
                        key,
                        e
                    )
                })?;
                Ok(())
            }
        }
    }

    async fn set_ex(&self, key: &str, value: &str, seconds: u64) -> anyhow::Result<()> {
        match &self.conn {
            RedisConnectionType::Single(conn) => {
                let mut conn = conn.clone();
                conn.set_ex(key, value, seconds)
                    .await
                    .map_err(|e: RedisError| anyhow!("[RedisRepositoryImpl::set_ex] Failed to set key '{}' with expiration: {:?}", key, e))?;
                Ok(())
            }
            RedisConnectionType::Cluster(conn) => {
                let mut conn = conn.clone();
                conn.set_ex(key, value, seconds)
                    .await
                    .map_err(|e: RedisError| anyhow!("[RedisRepositoryImpl::set_ex] Failed to set key '{}' with expiration: {:?}", key, e))?;
                Ok(())
            }
        }
    }

    async fn del(&self, key: &str) -> anyhow::Result<()> {
        match &self.conn {
            RedisConnectionType::Single(conn) => {
                let mut conn = conn.clone();
                conn.del(key).await.map_err(|e: RedisError| {
                    anyhow!(
                        "[RedisRepositoryImpl::del] Failed to delete key '{}': {:?}",
                        key,
                        e
                    )
                })?;
                Ok(())
            }
            RedisConnectionType::Cluster(conn) => {
                let mut conn = conn.clone();
                conn.del(key).await.map_err(|e: RedisError| {
                    anyhow!(
                        "[RedisRepositoryImpl::del] Failed to delete key '{}': {:?}",
                        key,
                        e
                    )
                })?;
                Ok(())
            }
        }
    }

    async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        match &self.conn {
            RedisConnectionType::Single(conn) => {
                let mut conn = conn.clone();
                let exists: bool = conn.exists(key).await.map_err(|e: RedisError| {
                    anyhow!(
                        "[RedisRepositoryImpl::exists] Failed to check key '{}': {:?}",
                        key,
                        e
                    )
                })?;
                Ok(exists)
            }
            RedisConnectionType::Cluster(conn) => {
                let mut conn = conn.clone();
                let exists: bool = conn.exists(key).await.map_err(|e: RedisError| {
                    anyhow!(
                        "[RedisRepositoryImpl::exists] Failed to check key '{}': {:?}",
                        key,
                        e
                    )
                })?;
                Ok(exists)
            }
        }
    }

    async fn expire(&self, key: &str, seconds: u64) -> anyhow::Result<()> {
        match &self.conn {
            RedisConnectionType::Single(conn) => {
                let mut conn = conn.clone();
                conn.expire(key, seconds as i64)
                    .await
                    .map_err(|e: RedisError| anyhow!("[RedisRepositoryImpl::expire] Failed to set expiration for key '{}': {:?}", key, e))?;
                Ok(())
            }
            RedisConnectionType::Cluster(conn) => {
                let mut conn = conn.clone();
                conn.expire(key, seconds as i64)
                    .await
                    .map_err(|e: RedisError| anyhow!("[RedisRepositoryImpl::expire] Failed to set expiration for key '{}': {:?}", key, e))?;
                Ok(())
            }
        }
    }
}
