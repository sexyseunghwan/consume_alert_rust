use crate::common::*;

use crate::service_traits::elastic_query_service::*;
use crate::service_traits::graph_api_service::*;
use crate::service_traits::mysql_query_service::*;
use crate::service_traits::process_service::*;
use crate::service_traits::producer_service::*;
use crate::service_traits::redis_service::*;
use crate::service_traits::telebot_service::*;

use crate::utils_modules::io_utils::*;
use crate::utils_modules::time_utils::*;

use crate::configuration::elasitc_index_name::*;

use crate::models::agg_result_set::*;
use crate::models::consume_index_prodt_type::*;
use crate::models::consume_result_by_type::*;
use crate::models::per_datetime::*;
use crate::models::spent_detail::*;
use crate::models::spent_detail_by_es::*;
use crate::models::spent_detail_to_kafka::*;
use crate::models::spent_detail_with_info::*;
use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;
use crate::models::user_payment_methods::*;

use crate::enums::{indexing_type::*, range_operator::*};

use crate::config::AppConfig;
use crate::views::spent_detail_view::SpentDetailView;

pub struct MainController<
    G: GraphApiService,
    E: ElasticQueryService,
    M: MysqlQueryService,
    T: TelebotService,
    P: ProcessService,
    KP: ProducerService,
    R: RedisService,
> {
    graph_api_service: Arc<G>,
    elastic_query_service: Arc<E>,
    mysql_query_service: Arc<M>,
    tele_bot_service: T,
    process_service: Arc<P>,
    producer_service: Arc<KP>,
    redis_service: Arc<R>,
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
    /// Returns `Ok(None)` when no user row exists for the given `user_seq`.
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
            .context("[resolve_user_id] Redis read failed")?
        {
            return Ok(Some(cached));
        }

        let user_id_opt: Option<String> = self
            .mysql_query_service
            .get_user_id_by_seq(user_seq)
            .await
            .context("[resolve_user_id] MySQL query failed")?;

        if let Some(ref user_id) = user_id_opt {
            self.redis_service
                .set_string(redis_key, user_id, None)
                .await
                .context("[resolve_user_id] Redis write failed")?;
        }

        Ok(user_id_opt)
    }

    /// Resolves `user_seq` via Redis cache, falling back to MySQL on a miss.
    ///
    /// Returns `Ok(None)` when the token / user_id pair is not registered.
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
            .context("[resolve_user_seq] Redis read failed")?
        {
            return Ok(Some(
                cached
                    .parse::<i64>()
                    .context("[resolve_user_seq] Failed to parse cached user_seq")?,
            ));
        }

        let seq_opt: Option<i64> = self
            .mysql_query_service
            .exists_telegram_room_by_token_and_id(telegram_token, telegram_user_id)
            .await
            .context("[resolve_user_seq] MySQL query failed")?;

        if let Some(seq) = seq_opt {
            self.redis_service
                .set_string(redis_key, &seq.to_string(), None)
                .await
                .context("[resolve_user_seq] Redis write failed")?;
        }

        Ok(seq_opt)
    }

    /// Resolves `room_seq` via Redis cache, falling back to MySQL on a miss.
    ///
    /// Returns `Ok(None)` when no room row exists for the given token + user.
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
            .context("[resolve_room_seq] Redis read failed")?
        {
            return Ok(Some(
                cached
                    .parse::<i64>()
                    .context("[resolve_room_seq] Failed to parse cached room_seq")?,
            ));
        }

        let seq_opt = self
            .mysql_query_service
            .get_telegram_room_seq_by_token_and_userseq(telegram_token, user_seq)
            .await
            .context("[resolve_room_seq] MySQL query failed")?;

        if let Some(seq) = seq_opt {
            self.redis_service
                .set_string(redis_key, &seq.to_string(), None)
                .await
                .context("[resolve_room_seq] Redis write failed")?;
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
    async fn resolve_spend_type(
        &self,
        spend_name: &str,
    ) -> anyhow::Result<ConsumingIndexProdtType> {
        let spent_type: ConsumingIndexProdtType = self
            .elastic_query_service
            .get_consume_type_judgement(spend_name)
            .await
            .context("[MainController::resolve_spend_type] Elasticsearch query failed")?;

        Ok(spent_type)
    }

    // ── Entry point ──────────────────────────────────────────────────────────

    /// Dispatches each incoming Telegram message to the matching command handler.
    ///
    /// ************ Main Function ************
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

        // let redis_user_id_key: String = format!("{}:{}", app_config.redis_user_id_key(), user_seq);
        // let _user_id: String = match self.resolve_user_id(&redis_user_id_key, user_seq).await? {
        //     Some(id) => id,
        //     None => {
        //         self.tele_bot_service
        //             .send_message_confirm(
        //                 "The token is invalid or you are not an authorized user.\nPlease contact the administrator.",
        //             )
        //             .await?;
        //         return Ok(());
        //     }
        // };

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
    fn preprocess_string(&self, delimiter: &str) -> Vec<String> {
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
    async fn common_process_python_double(
        &self,
        index_name: &str,
        permon_datetime: PerDatetime,
        start_op: RangeOperator,
        end_op: RangeOperator,
        room_seq: i64,
        detail_yn: bool,
    ) -> anyhow::Result<()> {
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

    // ── Command handlers ─────────────────────────────────────────────────────

    /// `c <name>:<amount>` — Records a single consumption entry manually.
    async fn command_consumption(
        &self,
        user_seq: i64,
        produce_topic: &str,
        room_seq: i64,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(":");

        if args.len() != 2 {
            self.tele_bot_service
                .send_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) c snack:15000",
                )
                .await?;
            return Err(anyhow!(
                "[main_controller::command_consumptio] Invalid parameter format: {}",
                self.tele_bot_service.get_input_text()
            ));
        }

        let spent_name: String = args[0].clone();
        let spent_money: i64 = match get_parsed_value_from_vector(&args, 1) {
            Ok(cash) => cash,
            Err(e) => {
                self.tele_bot_service
                    .send_message_confirm(
                        "The second parameter must be numeric.\nEX) c snack:15000",
                    )
                    .await?;
                return Err(anyhow!(
                    "[main_controller::command_consumptio] Non-numeric cash parameter: {:#}",
                    e
                ));
            }
        };

        let spent_type: ConsumingIndexProdtType = self
            .resolve_spend_type(&spent_name)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to insert to MySQL: {:#}",
                    e
                );
            })?;

        let default_payment_method: UserPaymentMethods = match self
            .mysql_query_service
            .get_user_payment_methods(user_seq, true)
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption] Failed to get user payment methods: {:#}", e);
            })?
            .first() {
                Some(default_payment_method) => default_payment_method.clone(),
                None => {
                    self.tele_bot_service
                    .send_message_confirm(
                        "Default payment method does not exist. \n
                        Please register a default payment method.",
                    )
                    .await?;
                    return Err(anyhow!("[main_controller::command_consumption] Default payment method does not exist."))
                }
            };

        let spent_detail: SpentDetail = SpentDetail::new(
            spent_name,
            spent_money,
            Local::now(),
            1,
            user_seq,
            0,
            spent_type.consume_keyword_type_id,
            room_seq,
            default_payment_method.payment_method_id,
        );

        let spent_detail_view: SpentDetailView = spent_detail
            .convert_spent_detail_to_view(&spent_type)
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to build view: {:#}",
                    e
                );
            })?;

        let spent_idx: i64 = self
            .mysql_query_service
            .insert_prodt_detail_with_transaction(&spent_detail)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to insert to MySQL: {:#}",
                    e
                );
            })?;

        let utc_now: DateTime<Utc> = Utc::now();

        let produce_payload: SpentDetailToKafka =
            SpentDetailToKafka::new(spent_idx, String::from("I"), utc_now);

        let partition_key: String = spent_idx.to_string();

        self.producer_service
            .produce_object_to_topic(
                produce_topic,
                &produce_payload,
                Some(partition_key.as_str()),
            )
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to produce Kafka message: {:#}",
                    e
                );
            })?;

        self.tele_bot_service
            .send_message_confirm(&spent_detail_view.to_telegram_string())
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to send Telegram message: {:#}",
                    e
                );
            })?;

        Ok(())
    }

    /// Auto-detects and records a consumption entry from a card-payment notification message.
    pub async fn command_consumption_auto(
        &self,
        user_seq: i64,
        produce_topic: &str,
        room_seq: i64,
    ) -> anyhow::Result<()> {
        let args: String = self.tele_bot_service.get_input_text();

        let bracket_re: Regex = Regex::new(r"\[.*?\]\n?").map_err(|e| {
            anyhow!(
                "[main_controller::command_consumption_auto] Bad regex: {:?}",
                e
            )
        })?;

        let lines: Vec<String> = bracket_re
            .replace_all(&args, "")
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Not a card-payment notification — silently ignore.
        if lines.is_empty() {
            return Ok(());
        }

        // 유저의 가능한 결제 정보를 가져와준다.
        let user_payment_methods: Vec<UserPaymentMethods> = self
            .mysql_query_service
            .get_user_payment_methods(user_seq, false)
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to get user payment methods: {:#}", e);
            })?;

        let mut spent_detail: SpentDetail = self
            .process_service
            .process_by_consume_filter(&lines, user_seq, room_seq, user_payment_methods)
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] {:#}", e);
            })?;

        // Clone the primary name to release the immutable borrow before iter_mut below.
        let primary_name: String = spent_detail.spent_name().to_string();

        let spent_type: ConsumingIndexProdtType = self
            .resolve_spend_type(&primary_name)
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to resolve spend type: {:#}", e);
            })?;

        spent_detail.set_consume_keyword_type_id(spent_type.consume_keyword_type_id);

        let spent_detail_view: SpentDetailView = spent_detail
            .convert_spent_detail_to_view(&spent_type)
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption_auto] Failed to build view: {:#}",
                    e
                );
            })?;

        let spent_idx: i64 = self
            .mysql_query_service
            .insert_prodt_detail_with_transaction(&spent_detail)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption_auto] Failed to insert to MySQL: {:#}",
                    e
                );
            })?;

        let utc_now: DateTime<Utc> = Utc::now();

        let produce_payload: SpentDetailToKafka =
            SpentDetailToKafka::new(spent_idx, String::from("I"), utc_now);

        let partition_key: String = spent_idx.to_string();

        self.producer_service
            .produce_object_to_topic(produce_topic, &produce_payload, Some(partition_key.as_str()))
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to produce Kafka message: {:#}", e);
            })?;

        self.tele_bot_service
            .send_message_confirm(&spent_detail_view.to_telegram_string())
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to send Telegram message: {:#}", e);
            })?;

        Ok(())
    }

    /// `cd` — Deletes the most recently recorded consumption entry.
    async fn command_delete_recent_consumption(
        &self,
        produce_topic: &str,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        if args.len() != 1 {
            self.tele_bot_service
                .send_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) cd",
                )
                .await?;
            return Ok(());
        }

        let latest_spent_detail: SpentDetailWithInfo = match self
            .mysql_query_service
            .get_latest_spent_detail(user_seq, room_seq)
            .await?
        {
            Some(latest_spent_detail) => latest_spent_detail,
            None => {
                self.tele_bot_service
                    .send_message_confirm("No expenses to delete.")
                    .await?;
                return Ok(());
            }
        };

        let spent_idx: i64 = latest_spent_detail.spent_idx;

        let spent_detail_view: SpentDetailView = latest_spent_detail.convert_to_view();

        match self
            .mysql_query_service
            .delete_spent_detail_with_transaction(spent_idx)
            .await
        {
            Ok(_) => {
                info!(
                    "[command_delete_recent_consumption] latest spent_idx={} (user_seq={}, room_seq={})",
                    spent_idx, user_seq, room_seq
                );

                self.tele_bot_service
                    .send_message_confirm(&spent_detail_view.to_telegram_string_to_delete())
                    .await?;

                let utc_now: DateTime<Utc> = Utc::now();

                let produce_payload: SpentDetailToKafka =
                    SpentDetailToKafka::new(spent_idx, String::from("D"), utc_now);

                let partition_key: String = spent_idx.to_string();

                self.producer_service
                    .produce_object_to_topic(produce_topic, &produce_payload, Some(partition_key.as_str()))
                    .await
                    .inspect_err(|e| {
                        error!("[main_controller::command_consumption_auto] Failed to produce Kafka message: {:#}", e);
                    })?;
            }
            Err(e) => {
                error!("[command_delete_recent_consumption] Failed delete SPENT_DETAIL information-{}: {:#}", spent_idx, e)
            }
        }

        Ok(())
    }

    /// `cm [YYYY.MM]` — Shows monthly consumption summary (current month if no arg).
    pub async fn command_consumption_per_mon(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let date_start: DateTime<Utc> = get_current_kor_naivedate_first_date()?;
                let date_end: DateTime<Utc> = get_lastday_naivedate(date_start)?;

                self.process_service
                    .get_nmonth_to_current_date(date_start, date_end, -1)?
            }
            2 if args
                .get(1)
                .is_some_and(|d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) =>
            {
                let parts: Vec<&str> = args[1].split('.').collect();
                let year: i32 = parts
                    .first()
                    .ok_or_else(|| anyhow!("[command_consumption_per_mon] Missing year"))?
                    .parse()?;
                let month: u32 = parts
                    .get(1)
                    .ok_or_else(|| anyhow!("[command_consumption_per_mon] Missing month"))?
                    .parse()?;
                let date_start: DateTime<Utc> = get_naivedate(year, month, 1)?;
                let date_end: DateTime<Utc> = get_lastday_naivedate(date_start)?;
                self.process_service
                    .get_nmonth_to_current_date(date_start, date_end, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "Invalid date format. Please use format YYYY.MM like cm 2023.07",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_mon] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            true,
        )
        .await
    }

    /// `ctr YYYY.MM.DD-YYYY.MM.DD` — Shows consumption summary for a custom date range.
    async fn command_consumption_per_term(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime = match args.len() {
            2 if args.get(1).is_some_and(|d| {
                validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}$")
                    .unwrap_or(false)
            }) =>
            {
                let parts: Vec<&str> = args[1].split('-').collect();
                let start_date: DateTime<Utc> = parse_date_as_utc_datetime(parts[0], "%Y.%m.%d")
                    .context("[command_consumption_per_term] Invalid start date format")?;
                let end_date: DateTime<Utc> = parse_date_as_utc_datetime(parts[1], "%Y.%m.%d")
                    .context("[command_consumption_per_term] Invalid end date format")?;
                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX) ctr 2023.07.07-2023.08.01",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_term] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            true,
        )
        .await
    }

    /// `ct [YYYY.MM.DD]` — Shows daily consumption summary (today if no arg).
    async fn command_consumption_per_day(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let today: DateTime<Utc> = get_current_kor_naivedate();
                self.process_service
                    .get_nday_to_current_date(today, today, -1)?
            }
            2 if args.get(1).is_some_and(|d| {
                validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}$").unwrap_or(false)
            }) =>
            {
                let date: DateTime<Utc> = parse_date_as_utc_datetime(&args[1], "%Y.%m.%d")
                    .context("[command_consumption_per_day] Invalid date format")?;
                self.process_service
                    .get_nday_to_current_date(date, date, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX) ct or ct 2023.11.11",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_day] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            true,
        )
        .await
    }

    /// `cw` — Shows consumption summary for the current week (Mon–Sun).
    async fn command_consumption_per_week(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let today: DateTime<Utc> = get_current_kor_naivedate();
                let days_to_monday: i64 = Weekday::Mon.num_days_from_monday() as i64
                    - today.weekday().num_days_from_monday() as i64;
                let monday: DateTime<Utc> = today + chrono::Duration::days(days_to_monday);
                let date_end: DateTime<Utc> = monday + chrono::Duration::days(6);
                self.process_service
                    .get_nday_to_current_date(monday, date_end, -7)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX) cw",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_week] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            true,
        )
        .await
    }

    /// `cy [YYYY]` — Shows yearly consumption summary (current year if no arg).
    async fn command_consumption_per_year(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let cur_year = get_current_kor_naivedate().year();
                let start_date: DateTime<Utc> = get_naivedate(cur_year, 1, 1)?;
                let end_date: DateTime<Utc> = get_naivedate(cur_year, 12, 31)?;
                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -12)?
            }
            2 if args
                .get(1)
                .is_some_and(|d| validate_date_format(d, r"^\d{4}$").unwrap_or(false)) =>
            {
                let year: i32 = args[1].parse()?;
                let start_date: DateTime<Utc> = get_naivedate(year, 1, 1)?;
                let end_date: DateTime<Utc> = get_naivedate(year, 12, 31)?;
                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -12)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX01) cy\nEX02) cy 2023",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_year] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            false,
        )
        .await
    }

    /// `cs [YYYY.MM]` — Shows consumption from the last payday (25th) to the next.
    pub async fn command_consumption_per_salary(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let today: DateTime<Utc> = get_current_kor_naivedate();
                let (year, month, day) = (today.year(), today.month(), today.day());
                let cur_date_start: DateTime<Utc> = if day < 25 {
                    get_add_month_from_naivedate(get_naivedate(year, month, 25)?, -1)?
                } else {
                    get_naivedate(year, month, 25)?
                };
                let cur_date_end: DateTime<Utc> = if day < 25 {
                    get_naivedate(year, month, 25)?
                } else {
                    get_add_month_from_naivedate(get_naivedate(year, month, 25)?, 1)?
                };
                self.process_service
                    .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
            }
            2 if args
                .get(1)
                .is_some_and(|d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) =>
            {
                let ref_date: DateTime<Utc> =
                    parse_date_as_utc_datetime(&format!("{}.01", args[1]), "%Y.%m.%d")
                        .context("[command_consumption_per_salary] Invalid date format")?;
                let cur_date_end: DateTime<Utc> =
                    get_naivedate(ref_date.year(), ref_date.month(), 25)?;
                let cur_date_start: DateTime<Utc> = get_add_month_from_naivedate(cur_date_end, -1)?;
                self.process_service
                    .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX) cs or cs 2023.11",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_salary] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThan,
            room_seq,
            true,
        )
        .await
    }
}
