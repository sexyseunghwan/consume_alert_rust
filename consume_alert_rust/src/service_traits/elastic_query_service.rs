use crate::common::*;

use crate::dtos::{EsRangeGroupSeqQueryDto, EsRangeRoomSeqQueryDto};
use crate::models::{agg_result_set::*, consume_index_prodt_type::*, document_with_id::*};

#[async_trait]
pub trait ElasticQueryService {
    async fn find_query_result_vec<T: DeserializeOwned>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error>;
    async fn find_consume_type_judgement(
        &self,
        prodt_name: &str,
    ) -> Result<ConsumingIndexProdtType, anyhow::Error>;
    #[allow(dead_code)]
    async fn find_info_orderby_cnt<T: DeserializeOwned>(
        &self,
        index_name: &str,
        order_by_field: &str,
        top_size: i64,
        asc_yn: bool,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error>;
    async fn find_info_filter_roomseq_orderby_aggs_range<T: Send + Sync + DeserializeOwned>(
        &self,
        query: EsRangeRoomSeqQueryDto,
    ) -> Result<AggResultSet<T>, anyhow::Error>;
    async fn find_info_filter_groupseq_orderby_aggs_range<T: Send + Sync + DeserializeOwned>(
        &self,
        query: EsRangeGroupSeqQueryDto,
    ) -> Result<AggResultSet<T>, anyhow::Error>;
    #[allow(dead_code)]
    async fn delete_es_doc<T: Send + Sync>(
        &self,
        index_name: &str,
        doc: &DocumentWithId<T>,
    ) -> Result<(), anyhow::Error>;
}
