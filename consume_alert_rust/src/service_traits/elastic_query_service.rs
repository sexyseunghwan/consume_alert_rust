use crate::common::*;

use crate::models::agg_result_set::*;
use crate::models::consume_index_prodt_type::*;
use crate::models::document_with_id::*;

use crate::enums::range_operator::*;

#[async_trait]
pub trait ElasticQueryService {
    async fn get_query_result_vec<T: DeserializeOwned>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error>;

    async fn get_consume_type_judgement(
        &self,
        prodt_name: &str,
    ) -> Result<ConsumingIndexProdtType, anyhow::Error>;
    async fn get_info_orderby_cnt<T: DeserializeOwned>(
        &self,
        index_name: &str,
        order_by_field: &str,
        top_size: i64,
        asc_yn: bool,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error>;
    #[allow(clippy::too_many_arguments)]
    async fn get_info_orderby_aggs_range<T: Send + Sync + DeserializeOwned>(
        &self,
        index_name: &str,
        range_field: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        start_op: RangeOperator,
        end_op: RangeOperator,
        order_by_field: &str,
        asc_yn: bool,
        aggs_field: &str,
    ) -> Result<AggResultSet<T>, anyhow::Error>;
    async fn delete_es_doc<T: Send + Sync>(
        &self,
        index_name: &str,
        doc: &DocumentWithId<T>,
    ) -> Result<(), anyhow::Error>;
}