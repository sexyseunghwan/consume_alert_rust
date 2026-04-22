use crate::common::*;

use crate::service_traits::elastic_query_service::*;
use crate::service_traits::graph_api_service::*;
use crate::service_traits::mysql_query_service::*;
use crate::service_traits::process_service::*;
use crate::service_traits::producer_service::*;
use crate::service_traits::redis_service::*;
use crate::service_traits::telebot_service::*;

use crate::models::agg_result_set::*;
use crate::models::consume_index_prodt_type::*;
use crate::models::consume_result_by_type::*;
use crate::models::per_datetime::*;
use crate::models::spent_detail_by_es::*;
use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;

use crate::enums::range_operator::*;

use crate::config::AppConfig;

mod command_consume;
mod command_query;

pub struct MainController<
    G: GraphApiService,
    E: ElasticQueryService,
    M: MysqlQueryService,
    T: TelebotService,
    P: ProcessService,
    KP: ProducerService,
    R: RedisService,
> {
    pub(super) graph_api_service: Arc<G>,
    pub(super) elastic_query_service: Arc<E>,
    pub(super) mysql_query_service: Arc<M>,
    pub(super) tele_bot_service: T,
    pub(super) process_service: Arc<P>,
    pub(super) producer_service: Arc<KP>,
    pub(super) redis_service: Arc<R>,
}

impl<
        G: GraphApiService,
        E: ElasticQueryService,
        M: MysqlQueryService,
        T: TelebotService,
        P: ProcessService,
        KP: ProducerService,
        R: RedisService,
    > MainController<G, E, M, T, P, KP, R>
{
    /// Creates a new `MainController` wiring all service dependencies together.
    ///
    /// # Arguments
    ///
    /// * `graph_api_service` - Service for calling the Python graph API
    /// * `elastic_query_service` - Service for querying Elasticsearch
    /// * `mysql_query_service` - Service for querying MySQL
    /// * `tele_bot_service` - Service for sending Telegram messages
    /// * `process_service` - Service for business-logic processing
    /// * `producer_service` - Service for producing Kafka messages
    /// * `redis_service` - Service for Redis cache operations
    ///
    /// # Returns
    ///
    /// Returns a new `MainController` instance.
    pub fn new(
        graph_api_service: Arc<G>,
        elastic_query_service: Arc<E>,
        mysql_query_service: Arc<M>,
        tele_bot_service: T,
        process_service: Arc<P>,
        producer_service: Arc<KP>,
        redis_service: Arc<R>,
    ) -> Self {
        Self {
            graph_api_service,
            elastic_query_service,
            mysql_query_service,
            tele_bot_service,
            process_service,
            producer_service,
            redis_service,
        }
    }

    // ── Cache-backed lookup helpers ──────────────────────────────────────────

    /// Resolves `user_id` via Redis cache, falling back to MySQL on a miss.
    ///
    /// # Arguments
    ///
    /// * `redis_key` - The Redis key used to look up the cached value
    /// * `user_seq` - The user sequence number to query from MySQL on a cache miss
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(user_id))` if found, or `Ok(None)` if no user row exists for the given `user_seq`.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis read/write or the MySQL query fails.
    #[allow(dead_code)]
    async fn resolve_user_id(
        &self,
        redis_key: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<String>> {
        if let Some(cached) = self
            .redis_service
            .get_string(redis_key)
            .await
            .inspect_err(|e| error!("[resolve_user_id] Redis read failed: {:#}", e))?
        {
            return Ok(Some(cached));
        }

        let user_id_opt: Option<String> = self
            .mysql_query_service
            .get_user_id_by_seq(user_seq)
            .await
            .inspect_err(|e| error!("[resolve_user_id] MySQL query failed: {:#}", e))?;

        if let Some(ref user_id) = user_id_opt {
            self.redis_service
                .set_string(redis_key, user_id, None)
                .await
                .inspect_err(|e| error!("[resolve_user_id] Redis write failed: {:#}", e))?;
        }

        Ok(user_id_opt)
    }

    /// Resolves `user_seq` via Redis cache, falling back to MySQL on a miss.
    ///
    /// # Arguments
    ///
    /// * `redis_key` - The Redis key used to look up the cached value
    /// * `telegram_token` - The Telegram bot token used to identify the room
    /// * `telegram_user_id` - The Telegram user ID to match against registered users
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(user_seq))` if found, or `Ok(None)` if the token / user_id pair is not registered.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis read/write, the cached value parse, or the MySQL query fails.
    async fn resolve_user_seq(
        &self,
        redis_key: &str,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        if let Some(cached) = self
            .redis_service
            .get_string(redis_key)
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
                .set_string(redis_key, &seq.to_string(), None)
                .await
                .inspect_err(|e| error!("[resolve_user_seq] Redis write failed: {:#}", e))?;
        }

        Ok(seq_opt)
    }

    /// Resolves `room_seq` via Redis cache, falling back to MySQL on a miss.
    ///
    /// # Arguments
    ///
    /// * `redis_key` - The Redis key used to look up the cached value
    /// * `telegram_token` - The Telegram bot token used to identify the room
    /// * `user_seq` - The user sequence number to match against registered rooms
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(room_seq))` if found, or `Ok(None)` if no room row exists for the given token + user.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis read/write, the cached value parse, or the MySQL query fails.
    async fn resolve_room_seq(
        &self,
        redis_key: &str,
        telegram_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>> {
        if let Some(cached) = self
            .redis_service
            .get_string(redis_key)
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
                .set_string(redis_key, &seq.to_string(), None)
                .await
                .inspect_err(|e| error!("[resolve_room_seq] Redis write failed: {:#}", e))?;
        }

        Ok(seq_opt)
    }

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

    // ── Entry point ──────────────────────────────────────────────────────────

    /// Dispatches each incoming Telegram message to the matching command handler.
    ///
    /// Resolves the caller's `user_seq` and `room_seq` from cache (or MySQL on miss),
    /// then routes to the appropriate command based on the first whitespace-delimited token
    /// of the input text. Unrecognised tokens fall through to the auto-consumption handler.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success. Returns early with `Ok(())` and sends an error message
    /// to the user if the token or user is not authorised.
    ///
    /// # Errors
    ///
    /// Returns an error if any downstream service call (Redis, MySQL, Telegram) fails.
    pub async fn main_call_function(&self) -> anyhow::Result<()> {
        let telegram_token: String = self.tele_bot_service.get_telegram_token();
        let telegram_user_id: String = self.tele_bot_service.get_telegram_user_id();
        let app_config: &AppConfig = AppConfig::global();

        let redis_user_key: String = format!(
            "{}:{}:{}",
            app_config.redis_user_key(),
            telegram_user_id,
            telegram_token
        );

        let redis_room_key: String = format!(
            "{}:{}:{}",
            app_config.redis_room_key(),
            telegram_user_id,
            telegram_token
        );

        let user_seq: i64 = match self
            .resolve_user_seq(&redis_user_key, &telegram_token, &telegram_user_id)
            .await?
        {
            Some(seq) => seq,
            None => {
                self.tele_bot_service
                    .send_message_confirm(
                        "The token is invalid or you are not an authorized user.\nPlease contact the administrator.",
                    )
                    .await?;
                return Ok(());
            }
        };

        let room_seq: i64 = match self
            .resolve_room_seq(&redis_room_key, &telegram_token, user_seq)
            .await?
        {
            Some(seq) => seq,
            None => {
                self.tele_bot_service
                    .send_message_confirm(
                        "The token is invalid or you are not an authorized user.\nPlease contact the administrator.",
                    )
                    .await?;
                return Ok(());
            }
        };

        let produce_topic: &str = &app_config.produce_topic;
        let input_text: String = self.tele_bot_service.get_input_text();

        match input_text.split_whitespace().next().unwrap_or("") {
            "c" => {
                self.command_consumption(user_seq, produce_topic, room_seq)
                    .await?
            }
            "cd" => {
                self.command_delete_recent_consumption(produce_topic, user_seq, room_seq)
                    .await?
            }
            "cm" => self.command_consumption_per_mon(room_seq).await?,
            "ctr" => self.command_consumption_per_term(room_seq).await?,
            "ct" => self.command_consumption_per_day(room_seq).await?,
            "cs" => self.command_consumption_per_salary(room_seq).await?,
            "cw" => self.command_consumption_per_week(room_seq).await?,
            "cy" => self.command_consumption_per_year(room_seq).await?,
            _ => {
                self.command_consumption_auto(user_seq, produce_topic, room_seq)
                    .await?
            }
        }

        Ok(())
    }

    // ── Shared helpers ───────────────────────────────────────────────────────

    /// Splits the raw command text (skipping the 2-char command prefix) by `delimiter`.
    ///
    /// # Arguments
    ///
    /// * `delimiter` - The string to split on after stripping the leading command token
    ///
    /// # Returns
    ///
    /// Returns a `Vec<String>` of trimmed, non-empty tokens parsed from the input text.
    pub(super) fn preprocess_string(&self, delimiter: &str) -> Vec<String> {
        let args: String = self.tele_bot_service.get_input_text();

        args.chars()
            .skip(2)
            .collect::<String>()
            .split(delimiter)
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// Fetches consumption data for the given period from Elasticsearch, renders graphs
    /// via the Python API, and sends all results to the Telegram chat room.
    ///
    /// # Arguments
    ///
    /// * `index_name` - The Elasticsearch index to query
    /// * `permon_datetime` - Date range for both the current and comparison periods
    /// * `start_op` - Range operator applied to the start of the date range
    /// * `end_op` - Range operator applied to the end of the date range
    /// * `room_seq` - The Telegram room sequence number to scope the query
    /// * `detail_yn` - When `true`, also sends the per-item detail message before the graphs
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after all messages and images have been sent and temp files deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the Elasticsearch query, graph API call, or Telegram send fails.
    pub(super) async fn common_process_python_double(
        &self,
        index_name: &str,
        permon_datetime: PerDatetime,
        start_op: RangeOperator,
        end_op: RangeOperator,
        room_seq: i64,
        detail_yn: bool,
    ) -> anyhow::Result<()> {
        use crate::utils_modules::io_utils::*;

        let spent_detail_info: AggResultSet<SpentDetailByEs> = self
            .elastic_query_service
            .get_info_orderby_aggs_range(
                index_name,
                "spent_at",
                permon_datetime.date_start,
                permon_datetime.date_end,
                start_op,
                end_op,
                "spent_at",
                true,
                "spent_money",
                room_seq,
            )
            .await?;

        let versus_spent_detail_info: AggResultSet<SpentDetailByEs> = self
            .elastic_query_service
            .get_info_orderby_aggs_range(
                index_name,
                "spent_at",
                permon_datetime.n_date_start,
                permon_datetime.n_date_end,
                start_op,
                end_op,
                "spent_at",
                true,
                "spent_money",
                room_seq,
            )
            .await?;

        let cur_python_graph_info: ToPythonGraphLine = ToPythonGraphLine::new(
            "cur",
            permon_datetime.date_start,
            permon_datetime.date_end,
            &spent_detail_info,
        )?;

        let versus_python_graph_info: ToPythonGraphLine = ToPythonGraphLine::new(
            "versus",
            permon_datetime.n_date_start,
            permon_datetime.n_date_end,
            &versus_spent_detail_info,
        )?;

        if detail_yn {
            self.tele_bot_service
                .send_message_consume_split(&cur_python_graph_info, spent_detail_info.source_list())
                .await?;
        }

        let consume_detail_img_path: String = self
            .graph_api_service
            .call_python_matplot_consume_detail_double(
                &cur_python_graph_info,
                &versus_python_graph_info,
            )
            .await?;

        let consume_result_by_type: Vec<ConsumeResultByType> = self
            .process_service
            .get_consumption_result_by_category(&spent_detail_info)?;

        let circle_graph: ToPythonGraphCircle = self
            .process_service
            .convert_consume_result_by_type_to_python_graph_circle(
                &consume_result_by_type,
                *spent_detail_info.agg_result(),
                permon_datetime.date_start,
                permon_datetime.date_end,
            )?;

        let circle_graph_path: String = self
            .graph_api_service
            .call_python_matplot_consume_type(&circle_graph)
            .await?;

        let img_files: Vec<String> = vec![consume_detail_img_path, circle_graph_path];

        self.tele_bot_service.send_photo_confirm(&img_files).await?;

        self.tele_bot_service
            .send_message_consume_info_by_typelist(
                &consume_result_by_type,
                permon_datetime.date_start,
                permon_datetime.date_end,
                *spent_detail_info.agg_result(),
            )
            .await?;

        delete_file(img_files)?;

        Ok(())
    }
}
