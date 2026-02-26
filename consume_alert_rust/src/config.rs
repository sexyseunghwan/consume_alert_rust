use once_cell::sync::OnceCell;
use std::env;

use crate::common::*;

/// Global configuration struct for environment variables
/// This struct is thread-safe and can be accessed from multiple threads concurrently
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct AppConfig {
    /// Telegram bot token list (parsed from comma-separated BOT_TOKENS env var).
    /// The application spawns one independent polling loop per token.
    pub bot_tokens: Vec<String>,
    /// User ID
    pub user_id: String,
    /// Kafka topic name for producing messages
    pub produce_topic: String,
    /// Kafka broker addresses
    pub kafka_brokers: String,
    /// MySQL host address
    pub mysql_host: String,
    /// Database connection URL
    pub database_url: String,
    /// Elasticsearch URL
    pub es_db_url: String,
    /// Elasticsearch username
    pub es_id: String,
    /// Elasticsearch password
    pub es_pw: String,

    pub redis_user_key: String,

    pub redis_room_key: String,
}

/// Global static instance of AppConfig
/// This is initialized once and can be safely accessed from multiple threads
static APP_CONFIG: normalOnceCell<AppConfig> = normalOnceCell::new();

impl AppConfig {
    /// Initialize the global configuration from environment variables
    /// This should be called once at application startup
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if initialization succeeds, Err with message if fails
    ///
    /// # Example
    /// ```
    /// use consume_alert_rust::config::AppConfig;
    ///
    /// fn main() {
    ///     AppConfig::init().expect("Failed to initialize config");
    ///     let config = AppConfig::global();
    ///     println!("Consume topic: {}", config.consume_topic);
    /// }
    /// ```
    pub fn init() -> Result<(), String> {
        dotenv::dotenv().ok();

        let config: AppConfig = AppConfig {
            bot_tokens: env::var("BOT_TOKENS")
                .map_err(|_| "BOT_TOKENS not found in environment".to_string())?
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            user_id: env::var("USER_ID")
                .map_err(|_| "USER_ID not found in environment".to_string())?,
            produce_topic: env::var("PRODUCE_TOPIC")
                .map_err(|_| "PRODUCE_TOPIC not found in environment".to_string())?,
            kafka_brokers: env::var("KAFKA_BROKERS")
                .map_err(|_| "KAFKA_BROKERS not found in environment".to_string())?,
            mysql_host: env::var("MY_SQL_HOST").unwrap_or_else(|_| "localhost:3306".to_string()),
            database_url: env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL not found in environment".to_string())?,
            es_db_url: env::var("ES_DB_URL")
                .map_err(|_| "ES_DB_URL not found in environment".to_string())?,
            es_id: env::var("ES_ID").map_err(|_| "ES_ID not found in environment".to_string())?,
            es_pw: env::var("ES_PW").map_err(|_| "ES_PW not found in environment".to_string())?,
            redis_user_key: env::var("REDIS_USER_KEY")
                .map_err(|_| "REDIS_USER_KEY not found in environment".to_string())?,
            redis_room_key: env::var("REDIS_ROOM_KEY")
                .map_err(|_| "REDIS_ROOM_KEY not found in environment".to_string())?,
        };

        APP_CONFIG
            .set(config)
            .map_err(|_| "AppConfig already initialized".to_string())
    }

    /// Get a reference to the global configuration
    ///
    /// # Panics
    /// Panics if the configuration has not been initialized with `AppConfig::init()`
    ///
    /// # Returns
    /// * `&'static AppConfig` - Reference to the global configuration
    ///
    /// # Example
    /// ```
    /// use consume_alert_rust::config::AppConfig;
    ///
    /// let config = AppConfig::global();
    /// println!("Topic: {}", config.consume_topic);
    /// ```
    pub fn global() -> &'static AppConfig {
        APP_CONFIG
            .get()
            .expect("AppConfig not initialized. Call AppConfig::init() first.")
    }

    /// Try to get a reference to the global configuration
    /// Returns None if not initialized yet
    ///
    /// # Returns
    /// * `Option<&'static AppConfig>` - Some if initialized, None otherwise
    pub fn try_global() -> Option<&'static AppConfig> {
        APP_CONFIG.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_access() {
        // Note: This test requires .env file to be present
        if AppConfig::try_global().is_none() {
            let _ = AppConfig::init();
        }

        let config: &AppConfig = AppConfig::global();
        assert!(!config.produce_topic.is_empty());
        assert!(!config.kafka_brokers.is_empty());
    }
}
