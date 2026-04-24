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
    async fn find_user_seq(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        let app_config: &AppConfig = AppConfig::global();

        let redis_key: String = format!(
            "{}:{}:{}",
            app_config.redis_user_key(),
            telegram_user_id,
            telegram_token
        );

        if let Some(cached) = self
            .redis_service
            .get_string(&redis_key)
            .await
            .inspect_err(|e| error!("[resolve_user_seq] Redis read failed: {:#}", e))?
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
            .exists_telegram_room_by_token_and_id(telegram_token, telegram_user_id)
            .await
            .inspect_err(|e| error!("[resolve_user_seq] MySQL query failed: {:#}", e))?;

        if let Some(seq) = seq_opt {
            self.redis_service
                .set_string(&redis_key, &seq.to_string(), None)
                .await
                .inspect_err(|e| error!("[resolve_user_seq] Redis write failed: {:#}", e))?;
        }

        Ok(seq_opt)
    }

    async fn find_telegram_room_seq(
        &self,
        user_seq: i64,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        let app_config: &AppConfig = AppConfig::global();

        let redis_key: String = format!(
            "{}:{}:{}",
            app_config.redis_room_key(),
            telegram_user_id,
            telegram_token
        );

        if let Some(cached) = self
            .redis_service
            .get_string(&redis_key)
            .await
            .inspect_err(|e| error!("[resolve_room_seq] Redis read failed: {:#}", e))?
        {
            return Ok(Some(cached.parse::<i64>().inspect_err(|e| {
                error!(
                    "[resolve_room_seq] Failed to parse cached room_seq: {:#}",
                    e
                )
            })?));
        }

        let seq_opt: Option<i64> = self
            .mysql_query_service
            .get_telegram_room_seq_by_token_and_userseq(telegram_token, user_seq)
            .await
            .inspect_err(|e| error!("[resolve_room_seq] MySQL query failed: {:#}", e))?;

        if let Some(seq) = seq_opt {
            self.redis_service
                .set_string(&redis_key, &seq.to_string(), None)
                .await
                .inspect_err(|e| error!("[resolve_room_seq] Redis write failed: {:#}", e))?;
        }

        Ok(seq_opt)
    }
}

// #[allow(dead_code)]
// async fn resolve_user_id(
//     &self,
//     redis_key: &str,
//     user_seq: i64,
// ) -> anyhow::Result<Option<String>> {
//     if let Some(cached) = self
//         .redis_service
//         .get_string(redis_key)
//         .await
//         .inspect_err(|e| error!("[resolve_user_id] Redis read failed: {:#}", e))?
//     {
//         return Ok(Some(cached));
//     }

//     let user_id_opt: Option<String> = self
//         .mysql_query_service
//         .get_user_id_by_seq(user_seq)
//         .await
//         .inspect_err(|e| error!("[resolve_user_id] MySQL query failed: {:#}", e))?;

//     if let Some(ref user_id) = user_id_opt {
//         self.redis_service
//             .set_string(redis_key, user_id, None)
//             .await
//             .inspect_err(|e| error!("[resolve_user_id] Redis write failed: {:#}", e))?;
//     }

//     Ok(user_id_opt)
// }

// /// Resolves `user_seq` via Redis cache, falling back to MySQL on a miss.
// ///
// /// # Arguments
// ///
// /// * `telegram_token` - The Telegram bot token used to identify the room
// /// * `telegram_user_id` - The Telegram user ID to match against registered users
// ///
// /// # Returns
// ///
// /// Returns `Ok(Some(user_seq))` if found, or `Ok(None)` if the token / user_id pair is not registered.
// ///
// /// # Errors
// ///
// /// Returns an error if the Redis read/write, the cached value parse, or the MySQL query fails.
// async fn resolve_user_seq(
//     &self,
//     telegram_token: &str,
//     telegram_user_id: &str,
// ) -> anyhow::Result<Option<i64>> {
//     let app_config: &AppConfig = AppConfig::global();

//     let redis_key: String = format!(
//         "{}:{}:{}",
//         app_config.redis_user_key(),
//         telegram_user_id,
//         telegram_token
//     );

//     if let Some(cached) = self
//         .redis_service
//         .get_string(&redis_key)
//         .await
//         .inspect_err(|e| error!("[resolve_user_seq] Redis read failed: {:#}", e))?
//     {
//         return Ok(Some(cached.parse::<i64>().inspect_err(|e| {
//             error!(
//                 "[resolve_user_seq] Failed to parse cached user_seq: {:#}",
//                 e
//             )
//         })?));
//     }

//     let seq_opt: Option<i64> = self
//         .mysql_query_service
//         .exists_telegram_room_by_token_and_id(telegram_token, telegram_user_id)
//         .await
//         .inspect_err(|e| error!("[resolve_user_seq] MySQL query failed: {:#}", e))?;

//     if let Some(seq) = seq_opt {
//         self.redis_service
//             .set_string(&redis_key, &seq.to_string(), None)
//             .await
//             .inspect_err(|e| error!("[resolve_user_seq] Redis write failed: {:#}", e))?;
//     }

//     Ok(seq_opt)
// }

// /// Resolves `room_seq` via Redis cache, falling back to MySQL on a miss.
// ///
// /// # Arguments
// ///
// /// * `redis_key` - The Redis key used to look up the cached value
// /// * `telegram_token` - The Telegram bot token used to identify the room
// /// * `user_seq` - The user sequence number to match against registered rooms
// ///
// /// # Returns
// ///
// /// Returns `Ok(Some(room_seq))` if found, or `Ok(None)` if no room row exists for the given token + user.
// ///
// /// # Errors
// ///
// /// Returns an error if the Redis read/write, the cached value parse, or the MySQL query fails.
// async fn resolve_room_seq(
//     &self,
//     telegram_user_id: &str,
//     telegram_token: &str,
//     user_seq: i64,
// ) -> anyhow::Result<Option<i64>> {
//     let app_config: &AppConfig = AppConfig::global();

//     let redis_key: String = format!(
//         "{}:{}:{}",
//         app_config.redis_room_key(),
//         telegram_user_id,
//         telegram_token
//     );

//     if let Some(cached) = self
//         .redis_service
//         .get_string(&redis_key)
//         .await
//         .inspect_err(|e| error!("[resolve_room_seq] Redis read failed: {:#}", e))?
//     {
//         return Ok(Some(cached.parse::<i64>().inspect_err(|e| {
//             error!(
//                 "[resolve_room_seq] Failed to parse cached room_seq: {:#}",
//                 e
//             )
//         })?));
//     }

//     let seq_opt: Option<i64> = self
//         .mysql_query_service
//         .get_telegram_room_seq_by_token_and_userseq(telegram_token, user_seq)
//         .await
//         .inspect_err(|e| error!("[resolve_room_seq] MySQL query failed: {:#}", e))?;

//     if let Some(seq) = seq_opt {
//         self.redis_service
//             .set_string(&redis_key, &seq.to_string(), None)
//             .await
//             .inspect_err(|e| error!("[resolve_room_seq] Redis write failed: {:#}", e))?;
//     }

//     Ok(seq_opt)
// }
//}
