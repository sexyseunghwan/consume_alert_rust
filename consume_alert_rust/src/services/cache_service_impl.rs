use crate::common::*;

use crate::service_traits::{cache_service::*, mysql_query_service::*, redis_service::*};

use crate::AppConfig;

#[derive(Debug, Getters, Clone, new)]
pub struct CacheServiceImpl<R: RedisService, M: MysqlQueryService> {
    pub redis_service: Arc<R>,
    pub mysql_query_service: Arc<M>,
}

#[async_trait]
impl<R, M> CacheService for CacheServiceImpl<R, M>
where
    R: RedisService + Sync + Send,
    M: MysqlQueryService + Sync + Send,
{
    /// Looks up the user sequence number for the given Telegram token and user ID,
    /// using Redis as a cache before falling back to MySQL.
    ///
    /// # Arguments
    ///
    /// * `telegram_token` - The Telegram bot token identifying the room
    /// * `telegram_user_id` - The Telegram user ID string
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(i64))` with the `user_seq` if found, or `Ok(None)` if no matching record exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis read, MySQL query, or Redis write fails.
    async fn find_user_seq(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        let app_config: &AppConfig = AppConfig::get_global();

        let redis_key: String = format!(
            "{}:{}:{}",
            app_config.redis_user_key(),
            telegram_user_id,
            telegram_token
        );

        if let Some(cached) = self
            .redis_service
            .find_string(&redis_key)
            .await
            .inspect_err(|e| error!("[CacheServiceImpl::find_user_seq] Redis read failed: {:#}", e))?
        {
            return Ok(Some(cached.parse::<i64>().inspect_err(|e| {
                error!(
                    "[resolve_user_seq] Failed to parse cached user_seq: {:#}",
                    e
                )
            })?));
        }

        let seq_opt: Option<i64> = self
            .mysql_query_service
            .has_telegram_room_by_token_and_id(telegram_token, telegram_user_id)
            .await
            .inspect_err(|e| error!("[CacheServiceImpl::find_user_seq] MySQL query failed: {:#}", e))?;

        if let Some(seq) = seq_opt {
            self.redis_service
                .input_string(&redis_key, &seq.to_string(), None)
                .await
                .inspect_err(|e| error!("[CacheServiceImpl::find_user_seq] Redis write failed: {:#}", e))?;
        }

        Ok(seq_opt)
    }

    /// Looks up the Telegram room sequence number for the given user and token,
    /// using Redis as a cache before falling back to MySQL.
    ///
    /// # Arguments
    ///
    /// * `user_seq` - The unique sequence number identifying the user
    /// * `telegram_token` - The Telegram bot token identifying the room
    /// * `telegram_user_id` - The Telegram user ID string
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(i64))` with the `room_seq` if found, or `Ok(None)` if no matching record exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis read, MySQL query, or Redis write fails.
    async fn find_telegram_room_seq(
        &self,
        user_seq: i64,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        let app_config: &AppConfig = AppConfig::get_global();

        let redis_key: String = format!(
            "{}:{}:{}",
            app_config.redis_room_key(),
            telegram_user_id,
            telegram_token
        );

        if let Some(cached) = self
            .redis_service
            .find_string(&redis_key)
            .await
            .inspect_err(|e| error!("[CacheServiceImpl::find_telegram_room_seq] Redis read failed: {:#}", e))?
        {
            return Ok(Some(cached.parse::<i64>().inspect_err(|e| {
                error!(
                    "[find_telegram_room_seq] Failed to parse cached room_seq: {:#}",
                    e
                )
            })?));
        }

        let seq_opt: Option<i64> = self
            .mysql_query_service
            .find_telegram_room_seq_by_token_and_userseq(telegram_token, user_seq)
            .await
            .inspect_err(|e| error!("[CacheServiceImpl::find_telegram_room_seq] MySQL query failed: {:#}", e))?;

        if let Some(seq) = seq_opt {
            self.redis_service
                .input_string(&redis_key, &seq.to_string(), None)
                .await
                .inspect_err(|e| error!("[CacheServiceImpl::find_telegram_room_seq] Redis write failed: {:#}", e))?;
        }

        Ok(seq_opt)
    }

    /// Looks up the Telegram group sequence number for the given user and token,
    /// using Redis as a cache before falling back to MySQL.
    ///
    /// # Arguments
    ///
    /// * `user_seq` - The unique sequence number identifying the user
    /// * `telegram_token` - The Telegram bot token identifying the room
    /// * `telegram_user_id` - The Telegram user ID string
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(i64))` with the `group_seq` if found, or `Ok(None)` if no matching group exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis read, MySQL query, or Redis write fails.
    async fn find_telegram_group_seq(
        &self,
        user_seq: i64,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>> {

        let app_config: &AppConfig = AppConfig::get_global();

        let redis_key: String = format!(
            "{}:{}:{}",
            app_config.redis_room_group_key(),
            telegram_user_id,
            telegram_token
        );

        if let Some(cached) = self
            .redis_service
            .find_string(&redis_key)
            .await
            .inspect_err(|e| error!("[CacheServiceImpl::find_telegram_group_seq] Redis read failed: {:#}", e))?
        {
            return Ok(Some(cached.parse::<i64>().inspect_err(|e| {
                error!(
                    "[CacheServiceImpl::find_telegram_group_seq] Failed to parse cached room_seq: {:#}",
                    e
                )
            })?));
        }

        let seq_opt: Option<i64> = self
            .mysql_query_service
            .find_telegram_group_seq_by_token_and_userseq(telegram_token, user_seq)
            .await
            .inspect_err(|e| error!("[CacheServiceImpl::find_telegram_group_seq] MySQL query failed: {:#}", e))?;

        if let Some(seq) = seq_opt {
            self.redis_service
                .input_string(&redis_key, &seq.to_string(), None)
                .await
                .inspect_err(|e| error!("[CacheServiceImpl::find_telegram_group_seq] Redis write failed: {:#}", e))?;
        }

        Ok(seq_opt)
    }
}