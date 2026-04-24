use crate::common::*;

use crate::models::agg_result_set::*;
use crate::models::consume_index_prodt_type::*;
use crate::models::document_with_id::*;

use crate::enums::range_operator::*;

#[async_trait]
pub trait ElasticQueryService {
    /// Deserializes an Elasticsearch response body into a vector of documents with their IDs and scores.
    ///
    /// # Arguments
    ///
    /// * `response_body` - The raw JSON response from Elasticsearch
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<DocumentWithId<T>>)` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the response body is malformed or deserialization fails.
    async fn find_query_result_vec<T: DeserializeOwned>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error>;

    /// Determines the consumption category for the given product name using Elasticsearch and the Levenshtein algorithm.
    ///
    /// # Arguments
    ///
    /// * `prodt_name` - The name or description of the spending item to classify
    ///
    /// # Returns
    ///
    /// Returns `Ok(ConsumingIndexProdtType)` with the best-matching category on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the Elasticsearch query fails.
    async fn find_consume_type_judgement(
        &self,
        prodt_name: &str,
    ) -> Result<ConsumingIndexProdtType, anyhow::Error>;

    /// Retrieves the top N documents from an index sorted by the specified field.
    ///
    /// # Arguments
    ///
    /// * `index_name` - The Elasticsearch index to query
    /// * `order_by_field` - The field name to sort by
    /// * `top_size` - The maximum number of documents to return
    /// * `asc_yn` - If `true`, sorts ascending; otherwise descending
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<DocumentWithId<T>>)` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the Elasticsearch query fails.
    #[allow(dead_code)]
    async fn find_info_orderby_cnt<T: DeserializeOwned>(
        &self,
        index_name: &str,
        order_by_field: &str,
        top_size: i64,
        asc_yn: bool,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error>;

    /// Queries documents within a date range, sorted by a field, and returns results with their sum aggregation.
    ///
    /// # Arguments
    ///
    /// * `index_name` - The Elasticsearch index to query
    /// * `range_field` - The field name to apply the date range filter on
    /// * `start_date` - The start of the date range
    /// * `end_date` - The end of the date range
    /// * `start_op` - The range operator for the start date (e.g., `gte`)
    /// * `end_op` - The range operator for the end date (e.g., `lte`)
    /// * `order_by_field` - The field name to sort results by
    /// * `asc_yn` - If `true`, sorts ascending; otherwise descending
    /// * `aggs_field` - The numeric field to aggregate with a sum
    /// * `room_seq` - The Telegram room sequence to filter by
    ///
    /// # Returns
    ///
    /// Returns `Ok(AggResultSet<T>)` containing the documents and the aggregated sum on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the Elasticsearch query fails.
    #[allow(clippy::too_many_arguments)]
    async fn find_info_orderby_aggs_range<T: Send + Sync + DeserializeOwned>(
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
        room_seq: i64,
    ) -> Result<AggResultSet<T>, anyhow::Error>;

    /// Deletes a specific document from an Elasticsearch index.
    ///
    /// # Arguments
    ///
    /// * `index_name` - The Elasticsearch index containing the document
    /// * `doc` - The document wrapper containing the ID to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the Elasticsearch delete operation fails.
    #[allow(dead_code)]
    async fn delete_es_doc<T: Send + Sync>(
        &self,
        index_name: &str,
        doc: &DocumentWithId<T>,
    ) -> Result<(), anyhow::Error>;
}
