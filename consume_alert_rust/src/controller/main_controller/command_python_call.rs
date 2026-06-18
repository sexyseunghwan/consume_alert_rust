use crate::common::*;
use crate::service_traits::{
    cache_service::*, elastic_query_service::*, graph_api_service::*, mysql_query_service::*,
    process_service::*, producer_service::*, redis_service::*, telebot_service::*,
};

use crate::dtos::*;
use crate::models::{
    agg_result_set::*, consume_result_by_type::*, document_with_id::*, file_info::*,
    spent_detail_by_es::*, spent_detail_by_es_kst::*, to_python_graph_circle::*,
    to_python_graph_line::*, per_datetime::*
};

use crate::enums::{
    range_operator::*, 
};

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
        dto: CommonProcessPythonDoubleDto,
    ) -> anyhow::Result<()> {
        let index_name: &str = dto.index_name.as_str();
        let permon_datetime: PerDatetime = dto.permon_datetime;
        let start_op: RangeOperator = dto.start_op;
        let end_op: RangeOperator = dto.end_op;
        let room_seq: Option<i64> = dto.room_seq;
        let group_seq: Option<i64> = dto.group_seq;
        let detail_yn: bool = dto.detail_yn;
        
        let (spent_detail_info, versus_spent_detail_info): (
            AggResultSet<SpentDetailByEs>,
            AggResultSet<SpentDetailByEs>,
        ) = match (room_seq, group_seq) {
            (Some(rs), _) => {
                let cur: AggResultSet<SpentDetailByEs> = self
                    .elastic_query_service
                    .find_info_filter_roomseq_orderby_aggs_range(EsRangeRoomSeqQueryDto {
                        index_name: index_name.to_string(),
                        range_field: "spent_at".to_string(),
                        start_date: permon_datetime.date_start,
                        end_date: permon_datetime.date_end,
                        start_op,
                        end_op,
                        order_by_field: "spent_at".to_string(),
                        asc_yn: true,
                        aggs_field: "spent_money".to_string(),
                        room_seq: rs,
                    })
                    .await?;
                let versus: AggResultSet<SpentDetailByEs> = self
                    .elastic_query_service
                    .find_info_filter_roomseq_orderby_aggs_range(EsRangeRoomSeqQueryDto {
                        index_name: index_name.to_string(),
                        range_field: "spent_at".to_string(),
                        start_date: permon_datetime.n_date_start,
                        end_date: permon_datetime.n_date_end,
                        start_op,
                        end_op,
                        order_by_field: "spent_at".to_string(),
                        asc_yn: true,
                        aggs_field: "spent_money".to_string(),
                        room_seq: rs,
                    })
                    .await?;
                (cur, versus)
            }
            (None, Some(gs)) => {
                let cur: AggResultSet<SpentDetailByEs> = self
                    .elastic_query_service
                    .find_info_filter_groupseq_orderby_aggs_range(EsRangeGroupSeqQueryDto {
                        index_name: index_name.to_string(),
                        range_field: "spent_at".to_string(),
                        start_date: permon_datetime.date_start,
                        end_date: permon_datetime.date_end,
                        start_op,
                        end_op,
                        order_by_field: "spent_at".to_string(),
                        asc_yn: true,
                        aggs_field: "spent_money".to_string(),
                        group_seq: gs,
                    })
                    .await?;
                let versus: AggResultSet<SpentDetailByEs> = self
                    .elastic_query_service
                    .find_info_filter_groupseq_orderby_aggs_range(EsRangeGroupSeqQueryDto {
                        index_name: index_name.to_string(),
                        range_field: "spent_at".to_string(),
                        start_date: permon_datetime.n_date_start,
                        end_date: permon_datetime.n_date_end,
                        start_op,
                        end_op,
                        order_by_field: "spent_at".to_string(),
                        asc_yn: true,
                        aggs_field: "spent_money".to_string(),
                        group_seq: gs,
                    })
                    .await?;
                (cur, versus)
            }
            (None, None) => {
                return Err(anyhow!(
                    "[common_process_python_double] room_seq and group_seq are both None"
                ))
            }
        };

        // Convert UTC to KST for display
        let spent_detail_info_kst: AggResultSet<SpentDetailByEsKst> = AggResultSet::new(
            *spent_detail_info.agg_result(),
            spent_detail_info
                .source_list()
                .iter()
                .map(|item| {
                    let source_kst = SpentDetailByEsKst {
                        spent_idx: item.source.spent_idx,
                        spent_name: item.source.spent_name.clone(),
                        spent_money: item.source.spent_money,
                        spent_at: item.source.spent_at.with_timezone(&Seoul),
                        created_at: item.source.created_at.with_timezone(&Seoul),
                        user_seq: item.source.user_seq,
                        consume_keyword_type_id: item.source.consume_keyword_type_id,
                        consume_keyword_type: item.source.consume_keyword_type.clone(),
                        room_seq: item.source.room_seq,
                        produced_at: item.source.produced_at.map(|dt| dt.with_timezone(&Seoul)),
                    };
                    DocumentWithId::new(item.id.clone(), item.score, source_kst)
                })
                .collect(),
        );

        let versus_spent_detail_info_kst: AggResultSet<SpentDetailByEsKst> = AggResultSet::new(
            *versus_spent_detail_info.agg_result(),
            versus_spent_detail_info
                .source_list()
                .iter()
                .map(|item| {
                    let source_kst = SpentDetailByEsKst {
                        spent_idx: item.source.spent_idx,
                        spent_name: item.source.spent_name.clone(),
                        spent_money: item.source.spent_money,
                        spent_at: item.source.spent_at.with_timezone(&Seoul),
                        created_at: item.source.created_at.with_timezone(&Seoul),
                        user_seq: item.source.user_seq,
                        consume_keyword_type_id: item.source.consume_keyword_type_id,
                        consume_keyword_type: item.source.consume_keyword_type.clone(),
                        room_seq: item.source.room_seq,
                        produced_at: item.source.produced_at.map(|dt| dt.with_timezone(&Seoul)),
                    };
                    crate::models::document_with_id::DocumentWithId::new(
                        item.id.clone(),
                        item.score,
                        source_kst,
                    )
                })
                .collect(),
        );

        let cur_python_graph_info: ToPythonGraphLine = ToPythonGraphLine::new(
            "cur",
            permon_datetime.date_start,
            permon_datetime.date_end,
            &spent_detail_info_kst,
        )?;

        let versus_python_graph_info: ToPythonGraphLine = ToPythonGraphLine::new(
            "versus",
            permon_datetime.n_date_start,
            permon_datetime.n_date_end,
            &versus_spent_detail_info_kst,
        )?;

        if detail_yn {
            self.tele_bot_service
                .input_message_consume_split(
                    &cur_python_graph_info,
                    spent_detail_info_kst.source_list(),
                )
                .await?;
        }

        let consume_detail_graph: Vec<u8> = self
            .graph_api_service
            .find_python_matplot_consume_detail_double(
                &cur_python_graph_info,
                &versus_python_graph_info,
            )
            .await?;
        let consume_detail_graph_img: FileInfo =
            FileInfo::new(String::from("consume_detail"), consume_detail_graph);

        let consume_result_by_type: Vec<ConsumeResultByType> = self
            .process_service
            .find_consumption_result_by_category(&spent_detail_info_kst)?;
        let circle_graph: ToPythonGraphCircle = self
            .process_service
            .to_python_graph_circle_by_consume_type(
                &consume_result_by_type,
                *spent_detail_info_kst.agg_result(),
                permon_datetime.date_start,
                permon_datetime.date_end,
            )?;
        let circle_graph_img: Vec<u8> = self
            .graph_api_service
            .find_python_matplot_consume_type(&circle_graph)
            .await?;
        let circle_img: FileInfo = FileInfo::new(String::from("consume_type"), circle_graph_img);

        let img_files: Vec<FileInfo> = vec![consume_detail_graph_img, circle_img];

        self.tele_bot_service.input_photo_confirm(img_files).await?;

        self.tele_bot_service
            .input_message_consume_info_by_typelist(
                &consume_result_by_type,
                permon_datetime.date_start,
                permon_datetime.date_end,
                *spent_detail_info_kst.agg_result(),
            )
            .await?;

        Ok(())
    }
}
