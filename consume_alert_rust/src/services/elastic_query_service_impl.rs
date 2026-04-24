use crate::common::*;

use crate::repository::es_repository::*;

use crate::models::agg_result_set::*;
use crate::models::consume_index_prodt_type::*;
use crate::models::document_with_id::*;
use crate::models::score_manager::*;

use crate::configuration::elasitc_index_name::*;

use crate::enums::range_operator::*;

use crate::service_traits::elastic_query_service::*;

#[derive(Debug, Getters, Clone, new)]
pub struct ElasticQueryServiceImpl<R: EsRepository> {
    elastic_conn: R,
}

#[async_trait]
impl<R: EsRepository + Sync + Send + std::fmt::Debug> ElasticQueryService
    for ElasticQueryServiceImpl<R>
{
    #[doc = "Functions that return queried results as vectors"]
    /// # Arguments
    /// * `response_body` - Querying Results
    ///
    /// # Returns
    /// * Result<Vec<T>, anyhow::Error>
    async fn get_query_result_vec<T: DeserializeOwned>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error> {
        let hits: &Value = &response_body["hits"]["hits"];

        let results: Vec<DocumentWithId<T>> = hits
            .as_array()
            .ok_or_else(|| anyhow!("[Error][get_query_result_vec()] 'hits' field is not an array"))?
            .iter()
            .map(|hit| {
                let id: &str = hit.get("_id").and_then(|id| id.as_str()).ok_or_else(|| {
                    anyhow!("[Error][get_query_result_vec()] Missing '_id' field")
                })?;

                let source: &Value = hit.get("_source").ok_or_else(|| {
                    anyhow!("[Error][get_query_result_vec()] Missing '_source' field")
                })?;

                let source: T = serde_json::from_value(source.clone()).map_err(|e| {
                    anyhow!(
                        "[Error][get_query_result_vec()] Failed to deserialize source: {:?}",
                        e
                    )
                })?;

                let score: f64 = hit
                    .get("_score")
                    .and_then(|score| score.as_f64())
                    .unwrap_or(0.0);

                Ok::<DocumentWithId<T>, anyhow::Error>(DocumentWithId {
                    id: id.to_string(),
                    score,
                    source,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    #[doc = "Function that classifies the consumption details provided as parameters into a specific consumption type"]
    /// # Arguments
    /// * `prodt_name` - consumtion name
    ///
    /// # Returns
    /// * Result<ConsumingIndexProdtType, anyhow::Error>
    async fn get_consume_type_judgement(
        &self,
        prodt_name: &str,
    ) -> Result<ConsumingIndexProdtType, anyhow::Error> {
        let es_query: Value = json!({
            "query": {
                "match": {
                    "consume_keyword": prodt_name
                }
            }
        });

        let response_body: Value = self
            .elastic_conn
            .get_search_query(&es_query, &CONSUME_TYPE)
            .await
            .map_err(|e| {
                anyhow!(
                    "[ElasticQueryServiceImpl::get_consume_type_judgement] response_body: {:?}",
                    e
                )
            })?;

        let results: Vec<DocumentWithId<ConsumingIndexProdtType>> = self
            .get_query_result_vec(&response_body)
            .await
            .map_err(|e| {
                anyhow!(
                    "[ElasticQueryServiceImpl::get_consume_type_judgement] results: {:?}",
                    e
                )
            })?;

        if results.is_empty() {
            return Ok(ConsumingIndexProdtType::new(
                21,
                String::from("etc"),
                prodt_name.to_string(),
                0,
            ));
        } else {
            let mut manager: ScoreManager<ConsumingIndexProdtType> =
                ScoreManager::<ConsumingIndexProdtType>::new();

            for consume_type in results {
                let keyword_weight: f64 = *consume_type.source().keyword_weight() as f64;
                let score: f64 = *consume_type.score() * -1.0 * keyword_weight;

                if !score.is_finite() {
                    return Err(anyhow!("[ElasticQueryServiceImpl::get_consume_type_judgement] Invalid score value: {}", score));
                }

                let score_i64: i64 = score as i64;
                let keyword: &str = consume_type.source.consume_keyword();

                /* Use the 'levenshtein' algorithm to determine word match */
                let word_dist: usize = levenshtein(keyword, prodt_name);
                let word_dist_i64: i64 = word_dist.try_into()?;
                manager.insert(word_dist_i64 + score_i64, consume_type.source);
            }

            let score_data_keyword: ScoredData<ConsumingIndexProdtType> = match manager.pop_lowest()
            {
                Some(score_data_keyword) => score_data_keyword,
                None => {
                    return Err(anyhow!("[ElasticQueryServiceImpl::get_consume_type_judgement] The mapped data for variable 'score_data_keyword' does not exist."));
                }
            };

            return Ok(score_data_keyword.data);
        }
    }

    #[doc = "Function that returns data by applying an order in a particular index"]
    /// # Arguments
    /// * `order_by_field` - Fields to sort
    /// * `top_size` - Number of data to return
    /// * `asc_yn` - It determines whether it is sequential or reverse alignment.
    ///
    /// # Returns
    /// * Result<String, anyhow::Error>
    async fn get_info_orderby_cnt<T: DeserializeOwned>(
        &self,
        index_name: &str,
        order_by_field: &str,
        top_size: i64,
        asc_yn: bool,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error> {
        let asc_fileter: &str = if asc_yn { "asc" } else { "desc" };

        let query: Value = json!({
            "sort": {
                order_by_field : asc_fileter
            },
            "size": top_size
        });

        let response_body: Value = self
            .elastic_conn
            .get_search_query(&query, index_name)
            .await?;

        let res: Vec<DocumentWithId<T>> = self.get_query_result_vec(&response_body).await?;

        Ok(res)
    }

    #[doc = "Functions that return aggregate result data for a particular index"]
    /// # Arguments
    /// * `index_name` - index name
    /// * `start_date` - start date
    /// * `end_date` - end date
    /// * `start_op` - Start date included
    /// * `end_op` - End date included
    /// * `order_by_field` - Field Name Targeted for 'order by'
    /// * `asc_yn` - ascending order or descending order
    /// * `aggs_field` - Name of the field to be aggregated
    ///
    /// # Returns
    /// * Result<AggResultSet<T>, anyhow::Error>
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
        room_seq: i64,
    ) -> Result<AggResultSet<T>, anyhow::Error> {
        let order_by_asc: &str = if asc_yn { "asc" } else { "desc" };

        let query: Value = json!({
            "size": 10000,
            "query": {
                "bool": {
                    "filter": [
                        {
                            "range": {
                                range_field: {
                                    start_op.as_str() : start_date.format("%Y-%m-%d").to_string(),
                                    end_op.as_str() : end_date.format("%Y-%m-%d").to_string()
                                }
                            }
                        },
                        {
                            "term": {
                                "room_seq": room_seq
                            }
                        }
                    ]
                }
            },
            "aggs": {
                "aggs_result": {
                    "sum": {
                        "field": aggs_field
                    }
                }
            },
            "sort": {
                order_by_field: { "order": order_by_asc }
            }
        });

        info!("{}", query.to_string());

        let response_body: Value = self
            .elastic_conn
            .get_search_query(&query, index_name)
            .await?;

        let agg_result: f64 = match &response_body["aggregations"]["aggs_result"]["value"].as_f64()
        {
            Some(agg_result) => *agg_result,
            None => {
                return Err(anyhow!(
                    "[Error][get_info_orderby_aggs_range()] 'agg_result' error"
                ))
            }
        };

        let consume_list: Vec<DocumentWithId<T>> =
            self.get_query_result_vec(&response_body).await?;

        let result: AggResultSet<T> = AggResultSet::new(agg_result, consume_list);

        Ok(result)
    }

    #[doc = "Functions that erase specific documents in the index"]
    /// # Arguments
    /// * `index_name` - index name
    /// * `doc` - Document Objects to Clear
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn delete_es_doc<T: Send + Sync>(
        &self,
        index_name: &str,
        doc: &DocumentWithId<T>,
    ) -> Result<(), anyhow::Error> {
        let doc_id: &String = doc.id();

        self.elastic_conn.delete_query(doc_id, index_name).await?;

        Ok(())
    }
}
