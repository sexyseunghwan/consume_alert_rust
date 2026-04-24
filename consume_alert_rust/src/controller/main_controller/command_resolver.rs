use crate::common::*;

use crate::service_traits::cache_service::*;
use crate::service_traits::elastic_query_service::*;
use crate::service_traits::graph_api_service::*;
use crate::service_traits::mysql_query_service::*;
use crate::service_traits::process_service::*;
use crate::service_traits::producer_service::*;
use crate::service_traits::redis_service::*;
use crate::service_traits::telebot_service::*;

use crate::models::consume_index_prodt_type::*;
use crate::models::per_datetime::*;
use crate::models::spent_detail::*;
use crate::models::spent_detail_to_kafka::*;
use crate::models::spent_detail_with_info::*;
use crate::models::user_payment_methods::*;

use crate::configuration::elasitc_index_name::*;
use crate::enums::range_operator::*;
use crate::utils_modules::time_utils::*;

use super::MainController;

impl<
        G: GraphApiService,
        E: ElasticQueryService,
        M: MysqlQueryService,
        T: TelebotService,
        P: ProcessService,
        KP: ProducerService,
        R: RedisService,
        C: CacheService,
    > MainController<G, E, M, T, P, KP, R, C>
{
    /// Determines the consumption category for the given spending name via Elasticsearch.
    ///
    /// # Arguments
    ///
    /// * `spend_name` - The name or description of the spending item to classify
    ///
    /// # Returns
    ///
    /// Returns `Ok(ConsumingIndexProdtType)` with the matched category on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the Elasticsearch query fails.
    pub(super) async fn resolve_spend_type(
        &self,
        spend_name: &str,
    ) -> anyhow::Result<ConsumingIndexProdtType> {
        let spent_type: ConsumingIndexProdtType = self
            .elastic_query_service
            .get_consume_type_judgement(spend_name)
            .await
            .inspect_err(|e| {
                error!(
                    "[MainController::resolve_spend_type] Elasticsearch query failed: {:#}",
                    e
                )
            })?;

        Ok(spent_type)
    }

    pub(super) async fn resolve_user_seq(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<i64> {
        match self
            .cache_service
            .find_user_seq(telegram_token, telegram_user_id)
            .await?
        {
            Some(seq) => Ok(seq),
            None => {
                self.tele_bot_service
                    .send_message_confirm(
                        "The token is invalid or you are not an authorized user.\nPlease contact the administrator.",
                    )
                    .await?;
                Err(anyhow!(
                    "[resolve_user_seq] Unauthorized user: telegram_user_id={}, telegram_token={}",
                    telegram_user_id,
                    telegram_token
                ))
            }
        }
    }

    pub(super) async fn resolve_telegram_room_seq(
        &self,
        user_seq: i64,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<i64> {
        match self
            .cache_service
            .find_telegram_room_seq(user_seq, telegram_token, telegram_user_id)
            .await?
        {
            Some(seq) => Ok(seq),
            None => {
                self.tele_bot_service
                    .send_message_confirm(
                        "The token is invalid or you are not an authorized user.\nPlease contact the administrator.",
                    )
                    .await?;
                Err(anyhow!(
                    "[resolve_telegram_room_seq] Unauthorized user: user_seq={}, telegram_token={}, telegram_user_id={}",
                    user_seq,
                    telegram_token,
                    telegram_user_id
                ))
            }
        }
    }
}
