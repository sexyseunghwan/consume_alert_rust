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

use crate::utils_modules::io_utils::*;


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
        room_seq: Option<i64>,
        group_seq: Option<i64>,
        detail_yn: bool,
    ) -> anyhow::Result<()> {
        

        let (spent_detail_info, versus_spent_detail_info): (
            AggResultSet<SpentDetailByEs>,
            AggResultSet<SpentDetailByEs>,
        ) = match (room_seq, group_seq) {
            (Some(rs), _) => {
                let cur: AggResultSet<SpentDetailByEs> = self
                    .elastic_query_service
                    .find_info_filter_roomseq_orderby_aggs_range(
                        index_name,
                        "spent_at",
                        permon_datetime.date_start,
                        permon_datetime.date_end,
                        start_op,
                        end_op,
                        "spent_at",
                        true,
                        "spent_money",
                        rs,
                    )
                    .await?;
                let versus: AggResultSet<SpentDetailByEs> = self
                    .elastic_query_service
                    .find_info_filter_roomseq_orderby_aggs_range(
                        index_name,
                        "spent_at",
                        permon_datetime.n_date_start,
                        permon_datetime.n_date_end,
                        start_op,
                        end_op,
                        "spent_at",
                        true,
                        "spent_money",
                        rs,
                    )
                    .await?;
                (cur, versus)
            }
            (None, Some(gs)) => {
                let cur: AggResultSet<SpentDetailByEs> = self
                    .elastic_query_service
                    .find_info_filter_groupseq_orderby_aggs_range(
                        index_name,
                        "spent_at",
                        permon_datetime.date_start,
                        permon_datetime.date_end,
                        start_op,
                        end_op,
                        "spent_at",
                        true,
                        "spent_money",
                        gs,
                    )
                    .await?;
                let versus: AggResultSet<SpentDetailByEs> = self
                    .elastic_query_service
                    .find_info_filter_groupseq_orderby_aggs_range(
                        index_name,
                        "spent_at",
                        permon_datetime.n_date_start,
                        permon_datetime.n_date_end,
                        start_op,
                        end_op,
                        "spent_at",
                        true,
                        "spent_money",
                        gs,
                    )
                    .await?;
                (cur, versus)
            }
            (None, None) => {
                return Err(anyhow!(
                    "[common_process_python_double] room_seq and group_seq are both None"
                ))
            }
        };

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



    