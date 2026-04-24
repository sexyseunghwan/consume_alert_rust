use crate::common::*;

use crate::service_traits::cache_service::*;
use crate::service_traits::elastic_query_service::*;
use crate::service_traits::graph_api_service::*;
use crate::service_traits::mysql_query_service::*;
use crate::service_traits::process_service::*;
use crate::service_traits::producer_service::*;
use crate::service_traits::redis_service::*;
use crate::service_traits::telebot_service::*;

use crate::models::agg_result_set::*;
use crate::models::consume_result_by_type::*;
use crate::models::per_datetime::*;
use crate::models::spent_detail_by_es::*;
use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;

use crate::enums::range_operator::*;


mod command_consume;
mod command_query;
mod command_resolver;

pub struct MainController<
    G: GraphApiService,
    E: ElasticQueryService,
    M: MysqlQueryService,
    T: TelebotService,
    P: ProcessService,
    KP: ProducerService,
    R: RedisService,
    C: CacheService,
> {
    pub(super) graph_api_service: Arc<G>,
    pub(super) elastic_query_service: Arc<E>,
    pub(super) mysql_query_service: Arc<M>,
    pub(super) tele_bot_service: T,
    pub(super) process_service: Arc<P>,
    pub(super) producer_service: Arc<KP>,
    pub(super) redis_service: Arc<R>,
    pub(super) cache_service: Arc<C>,
}

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
    /// * `cache_service` - Service for cached user and room lookups
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
        cache_service: Arc<C>,
    ) -> Self {
        Self {
            graph_api_service,
            elastic_query_service,
            mysql_query_service,
            tele_bot_service,
            process_service,
            producer_service,
            redis_service,
            cache_service,
        }
    }

    /// Dispatches the current Telegram input to the matching command handler.
    ///
    /// Reads the bot token, Telegram user id, and raw input text from `tele_bot_service`,
    /// then routes to the appropriate handler based on the first whitespace-delimited token.
    /// Authentication and room resolution are performed inside each command handler.
    /// Unrecognised input falls through to the auto-consumption parser.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the selected handler completes successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the Telegram context or executing the selected handler fails.
    pub async fn main_call_function(&self) -> anyhow::Result<()> {
        let telegram_token: String = self.tele_bot_service.get_telegram_token();
        let telegram_user_id: String = self.tele_bot_service.get_telegram_user_id();
        let input_text: String = self.tele_bot_service.get_input_text();

        match input_text.split_whitespace().next().unwrap_or("") {
            "c" => {
                self.command_consumption(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cd" => {
                self.command_delete_recent_consumption(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cm" => {
                self.command_consumption_per_mon(&telegram_token, &telegram_user_id)
                    .await?
            }
            "ctr" => {
                self.command_consumption_per_term(&telegram_token, &telegram_user_id)
                    .await?
            }
            "ct" => {
                self.command_consumption_per_day(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cs" => {
                self.command_consumption_per_salary(&telegram_token, &telegram_user_id)
                    .await?
            }
            "sg" => {
                self.command_consumption_per_salary_group(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cw" => {
                self.command_consumption_per_week(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cy" => {
                self.command_consumption_per_year(&telegram_token, &telegram_user_id)
                    .await?
            }
            _ => {
                self.command_consumption_auto(&telegram_token, &telegram_user_id)
                    .await?
            } 
        }

        Ok(())
    }

    // ── Shared helpers ───────────────────────────────────────────────────────

    /// Splits the raw command text after dropping its first two characters.
    ///
    /// # Arguments
    ///
    /// * `delimiter` - The string to split on after trimming the leading two characters
    ///
    /// # Returns
    ///
    /// Returns a `Vec<String>` of trimmed tokens parsed from the input text.
    /// Empty tokens are preserved when the split operation yields them.
    pub(super) fn to_preprocessed_tokens(&self, delimiter: &str) -> Vec<String> {
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
            .find_info_orderby_aggs_range(
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
            .find_info_orderby_aggs_range(
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
                .input_message_consume_split(&cur_python_graph_info, spent_detail_info.source_list())
                .await?;
        }

        let consume_detail_img_path: String = self
            .graph_api_service
            .find_python_matplot_consume_detail_double(
                &cur_python_graph_info,
                &versus_python_graph_info,
            )
            .await?;

        let consume_result_by_type: Vec<ConsumeResultByType> = self
            .process_service
            .find_consumption_result_by_category(&spent_detail_info)?;

        let circle_graph: ToPythonGraphCircle = self
            .process_service
            .to_python_graph_circle_by_consume_type(
                &consume_result_by_type,
                *spent_detail_info.agg_result(),
                permon_datetime.date_start,
                permon_datetime.date_end,
            )?;

        let circle_graph_path: String = self
            .graph_api_service
            .find_python_matplot_consume_type(&circle_graph)
            .await?;

        let img_files: Vec<String> = vec![consume_detail_img_path, circle_graph_path];

        self.tele_bot_service.input_photo_confirm(&img_files).await?;

        self.tele_bot_service
            .input_message_consume_info_by_typelist(
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
