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
use crate::models::consume_prodt_info::*;
use crate::models::consume_result_by_type::*;
use crate::models::document_with_id::*;
use crate::models::per_datetime::*;
use crate::models::spent_detail::*;
use crate::models::spent_detail_by_installment::*;
use crate::models::spent_detail_by_produce::*;
use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;
use crate::models::spent_detail_by_es::*;

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

        let redis_user_id_key: String = format!(
            "{}:{}",
            app_config.redis_user_id_key(),
            user_seq
        );

        let user_id: String = match self
            .resolve_user_id(&redis_user_id_key, user_seq)
            .await?
        {
            Some(id) => id,
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
                self.command_consumption(user_seq, produce_topic, room_seq, &user_id)
                    .await?
            }
            "cd" => self.command_delete_recent_consumption().await?, // 일단 보류...
            "cm" => self.command_consumption_per_mon().await?,
            "ctr" => self.command_consumption_per_term().await?,
            "ct" => self.command_consumption_per_day().await?,
            "cs" => self.command_consumption_per_salary().await?,
            "cw" => self.command_consumption_per_week().await?,
            "cy" => self.command_consumption_per_year().await?,
            _ => {
                self.command_consumption_auto(user_seq, produce_topic, room_seq, &user_id)
                    .await?
            }
        }
        
        Ok(())
    }
    
    // ── Shared helpers ───────────────────────────────────────────────────────

    /// Splits the raw command text (skipping the 2-char command prefix) by `delimiter`.
    fn preprocess_string(&self, delimiter: &str) -> Vec<String> {
        let args: String = self.tele_bot_service.get_input_text();
        args[2..]
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
            )
            .await?;
            
        let cur_python_graph: ToPythonGraphLine = ToPythonGraphLine::new(
            "cur",
            permon_datetime.date_start,
            permon_datetime.date_end,
            &spent_detail_info,
        )?;

        let versus_python_graph: ToPythonGraphLine = ToPythonGraphLine::new(
            "versus",
            permon_datetime.n_date_start,
            permon_datetime.n_date_end,
            &versus_spent_detail_info,
        )?;

        self.tele_bot_service
            .send_message_consume_split(&cur_python_graph, spent_detail_info.source_list())
            .await?;

        let consume_detail_img_path: String = self
            .graph_api_service
            .call_python_matplot_consume_detail_double(&cur_python_graph, &versus_python_graph)
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
        user_id: &str
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(":");

        if args.len() != 2 {
            self.tele_bot_service
                .send_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) c snack:15000",
                )
                .await?;
            return Err(anyhow!(
                "[command_consumption] Invalid parameter format: {}",
                self.tele_bot_service.get_input_text()
            ));
        }

        let consume_name: &str = &args[0];
        let consume_cash: i64 = match get_parsed_value_from_vector(&args, 1) {
            Ok(cash) => cash,
            Err(e) => {
                self.tele_bot_service
                    .send_message_confirm(
                        "The second parameter must be numeric.\nEX) c snack:15000",
                    )
                    .await?;
                return Err(e).context("[command_consumption] Non-numeric cash parameter");
            }
        };

        let spent_type: ConsumingIndexProdtType =  self
            .resolve_spend_type(consume_name)
            .await
            .context("[command_consumption_auto] Failed to resolve spend type")?;

        // let (spent_type, spent_type_nm) = self
        //     .resolve_spend_type(consume_name)
        //     .await
        //     .context("[command_consumption] Failed to resolve spend type")?;
        
        let spent_detail: SpentDetail = SpentDetail::new(
            consume_name.to_string(),
            consume_cash,
            Local::now(),
            1,
            user_seq,
            1,
            spent_type.consume_keyword_type_id,
            room_seq
        );

        let spent_detail_view: SpentDetailView = spent_detail
            .convert_spent_detail_to_view(&spent_type)
            .context("[command_consumption] Failed to build view")?;

        let spent_idx: i64 = self
            .mysql_query_service
            .insert_prodt_detail_with_transaction(&spent_detail)
            .await
            .context("[command_consumption] Failed to insert to MySQL")?;

        let produce_payload: SpentDetailByProduce = spent_detail
            .convert_to_spent_detail_by_produce(
                spent_idx,
                spent_type.consume_keyword_type(),
                room_seq,
                IndexingType::Insert,
                user_id
            );

        self.producer_service
            .produce_object_to_topic(produce_topic, &produce_payload, None)
            .await
            .context("[command_consumption] Failed to produce Kafka message")?;
        
        self.tele_bot_service
            .send_message_struct_info(&spent_detail_view)
            .await
            .context("[command_consumption] Failed to send Telegram message")?;

        Ok(())
    }

    /// Auto-detects and records a consumption entry from a card-payment notification message.
    pub async fn command_consumption_auto(
        &self,
        user_seq: i64,
        produce_topic: &str,
        room_seq: i64,
        user_id: &str
    ) -> anyhow::Result<()> {
        let args: String = self.tele_bot_service.get_input_text();

        let bracket_re: Regex = Regex::new(r"\[.*?\]\n?")
            .map_err(|e| anyhow!("[command_consumption_auto] Bad regex: {:?}", e))?;

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

        let spent_detail_by_installment: SpentDetailByInstallment = self
            .process_service
            .process_by_consume_filter(&lines, user_seq, room_seq)
            .context("[command_consumption_auto] Failed to parse card notification")?;

        let mut spent_details: Vec<SpentDetail> = self
            .process_service
            .get_spent_detail_installment_process(&spent_detail_by_installment)?;

        // Clone the primary name to release the immutable borrow before iter_mut below.
        let primary_name: String = spent_details
            .first()
            .ok_or_else(|| anyhow!("[command_consumption_auto] spent_details is empty"))?
            .spent_name()
            .clone();
        
        let spent_type: ConsumingIndexProdtType =  self
            .resolve_spend_type(&primary_name)
            .await
            .context("[command_consumption_auto] Failed to resolve spend type")?;

        let spent_detail_views: Vec<SpentDetailView> = spent_details
            .iter_mut()
            .map(|detail| {
                detail.set_consume_keyword_type_id(*spent_type.consume_keyword_type_id());
                detail
                    .convert_spent_detail_to_view(&spent_type)
                    .context("[command_consumption_auto] Failed to build view")
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let inserted_idxs: Vec<i64> = self
            .mysql_query_service
            .insert_prodt_details_with_transaction(&spent_details)
            .await
            .context("[command_consumption_auto] Failed to insert to MySQL")?;

        // `inserted_idxs[i]` is the DB-assigned primary key for `spent_details[i]`.
        let produce_payloads: Vec<SpentDetailByProduce> = inserted_idxs
            .iter()
            .copied()
            .zip(spent_details.iter())
            .map(|(spent_idx, detail)| {
                detail.convert_to_spent_detail_by_produce(
                    spent_idx,
                    spent_type.consume_keyword_type(),
                    room_seq,
                    IndexingType::Insert,
                    user_id
                )
            })
            .collect();

        self.producer_service
            .produce_objects_to_topic(
                produce_topic,
                &produce_payloads,
                None::<fn(&SpentDetailByProduce) -> String>,
            )
            .await
            .context("[command_consumption_auto] Failed to produce Kafka messages")?;

        self.tele_bot_service
            .send_message_struct_list(&spent_detail_views)
            .await
            .context("[command_consumption_auto] Failed to send Telegram message")?;

        Ok(())
    }

    /// `cd` — Deletes the most recently recorded consumption entry.
    async fn command_delete_recent_consumption(&self) -> anyhow::Result<()> {
        let args = self.preprocess_string(" ");

        if args.len() != 1 {
            self.tele_bot_service
                .send_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) cd",
                )
                .await?;
            return Ok(());
        }

        let recent: Vec<DocumentWithId<ConsumeProdtInfo>> = self
            .elastic_query_service
            .get_info_orderby_cnt(&CONSUME_DETAIL, "cur_timestamp", 1, false)
            .await?;

        let top = recent.first().ok_or_else(|| {
            anyhow!("[command_delete_recent_consumption] No consumption records found")
        })?;

        self.elastic_query_service
            .delete_es_doc(&CONSUME_DETAIL, top)
            .await?;

        self.tele_bot_service
            .send_message_struct_info(top.source())
            .await?;

        Ok(())
    }

    /// `cm [YYYY.MM]` — Shows monthly consumption summary (current month if no arg).
    pub async fn command_consumption_per_mon(&self) -> anyhow::Result<()> {       
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let date_start: DateTime<Utc> = get_current_kor_naivedate_first_date()?;
                let date_end: DateTime<Utc> = get_lastday_naivedate(date_start)?;

                self.process_service
                    .get_nmonth_to_current_date(date_start, date_end, -1)?
            }
            2 if args.get(1).is_some_and(|d| {
                validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)
            }) =>
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
        )
        .await
    }

    /// `ctr YYYY.MM.DD-YYYY.MM.DD` — Shows consumption summary for a custom date range.
    async fn command_consumption_per_term(&self) -> anyhow::Result<()> {
        let args = self.preprocess_string(" ");

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
        )
        .await
    }

    /// `ct [YYYY.MM.DD]` — Shows daily consumption summary (today if no arg).
    async fn command_consumption_per_day(&self) -> anyhow::Result<()> {
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
        )
        .await
    }

    /// `cw` — Shows consumption summary for the current week (Mon–Sun).
    async fn command_consumption_per_week(&self) -> anyhow::Result<()> {
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
        )
        .await
    }

    /// `cy [YYYY]` — Shows yearly consumption summary (current year if no arg).
    async fn command_consumption_per_year(&self) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let cur_year = get_current_kor_naivedate().year();
                let start_date: DateTime<Utc> = get_naivedate(cur_year, 1, 1)?;
                let end_date: DateTime<Utc> = get_naivedate(cur_year, 12, 31)?;
                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -12)?
            }
            2 if args.get(1).is_some_and(|d| {
                validate_date_format(d, r"^\d{4}$").unwrap_or(false)
            }) =>
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
        )
        .await
    }

    /// `cs [YYYY.MM]` — Shows consumption from the last payday (25th) to the next.
    pub async fn command_consumption_per_salary(&self) -> anyhow::Result<()> {
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
            2 if args.get(1).is_some_and(|d| {
                validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)
            }) =>
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
        )
        .await
    }
}
