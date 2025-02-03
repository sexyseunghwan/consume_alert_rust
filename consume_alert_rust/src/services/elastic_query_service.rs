use crate::common::*;

use crate::utils_modules::time_utils::*;

use crate::repository::es_repository::*;

use crate::models::agg_result_set::*;
use crate::models::consume_index_prodt_type::*;
use crate::models::document_with_id::*;
use crate::models::score_manager::*;

use crate::configuration::elasitc_index_name::*;

#[async_trait]
pub trait ElasticQueryService {
    async fn get_query_result_vec<T: DeserializeOwned>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error>;

    async fn get_consume_type_judgement(&self, prodt_name: &str) -> Result<String, anyhow::Error>;
    async fn get_info_orderby_cnt<T: DeserializeOwned>(
        &self,
        index_name: &str,
        order_by_field: &str,
        top_size: i64,
        asc_yn: bool,
    ) -> Result<Vec<DocumentWithId<T>>, anyhow::Error>;
    async fn get_info_orderby_aggs_range<T: Send + Sync + DeserializeOwned>(
        &self,
        index_name: &str,
        range_field: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        order_by_field: &str,
        asc_yn: bool,
        aggs_field: &str,
    ) -> Result<AggResultSet<T>, anyhow::Error>;
    // async fn get_distinct_field_values(
    //     &self,
    //     index_name: &str,
    //     field_name: &str,
    // ) -> Result<Vec<DistinctObject>, anyhow::Error>; -> 필요없어 보임.
    async fn delete_es_doc<T: Send + Sync>(
        &self,
        index_name: &str,
        doc: &DocumentWithId<T>,
    ) -> Result<(), anyhow::Error>;

    //async fn get_versus_consume_detail_infos(&self, permon_datetime: PerDatetime) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct ElasticQueryServicePub;

#[async_trait]
impl ElasticQueryService for ElasticQueryServicePub {
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
    /// * Result<String, anyhow::Error>
    async fn get_consume_type_judgement(&self, prodt_name: &str) -> Result<String, anyhow::Error> {
        let es_client: EsRepositoryPub = get_elastic_conn()?;

        let es_query: Value = json!({
            "query": {
                "match": {
                    "consume_keyword": prodt_name
                }
            }
        });

        let response_body: Value = es_client.get_search_query(&es_query, &CONSUME_TYPE).await?;
        let results: Vec<DocumentWithId<ConsumingIndexProdtType>> =
            self.get_query_result_vec(&response_body).await?;

        if results.len() == 0 {
            return Ok(String::from("etc"));
        } else {
            let mut manager: ScoreManager<ConsumingIndexProdtType> =
                ScoreManager::<ConsumingIndexProdtType>::new();

            for consume_type in results {
                let score: f64 = *consume_type.score() * -1.0;
                let score_i64: i64 = score as i64;
                let keyword: &String = consume_type.source.consume_keyword();

                /* Use the 'levenshtein' algorithm to determine word match */
                let word_dist: usize = levenshtein(keyword, &prodt_name);
                let word_dist_i64: i64 = word_dist.try_into()?;
                manager.insert(word_dist_i64 + score_i64, consume_type.source);
            }

            let score_data_keyword: ScoredData<ConsumingIndexProdtType> = match manager.pop_lowest()
            {
                Some(score_data_keyword) => score_data_keyword,
                None => {
                    return Err(anyhow!("[Error][get_consume_prodt_details_specify_type()] The mapped data for variable 'score_data_keyword' does not exist."));
                }
            };

            let prodt_type: String = score_data_keyword.data().consume_keyword_type().to_string();

            return Ok(prodt_type);
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
        let es_client: EsRepositoryPub = get_elastic_conn()?;

        let asc_fileter: &str = if asc_yn { "asc" } else { "desc" };

        let query: Value = json!({
            "sort": {
                order_by_field : asc_fileter
            },
            "size": top_size
        });

        let response_body: Value = es_client.get_search_query(&query, index_name).await?;

        let res: Vec<DocumentWithId<T>> = self.get_query_result_vec(&response_body).await?;

        Ok(res)
    }

    #[doc = "Functions that return aggregate result data for a particular index"]
    /// # Arguments
    /// * `index_name` - index name
    /// * `start_date` - start date
    /// * `end_date` - end date
    /// * `order_by_field` - Field Name Targeted for 'order by'
    /// * `asc_yn` - ascending order or descending order
    /// * `aggs_field` - Name of the field to be aggregated
    ///
    /// # Returns
    /// * Result<AggResultSet<T>, anyhow::Error>
    async fn get_info_orderby_aggs_range<T: Send + Sync + DeserializeOwned>(
        &self,
        index_name: &str,
        range_field: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        order_by_field: &str,
        asc_yn: bool,
        aggs_field: &str,
    ) -> Result<AggResultSet<T>, anyhow::Error> {
        let es_client: EsRepositoryPub = get_elastic_conn()?;

        let order_by_asc: &str = if asc_yn { "asc" } else { "desc" };

        let query: Value = json!({
            "size": 10000,
            "query": {
                "range": {
                    range_field: {
                        "gte": get_str_from_naivedate(start_date),
                        "lt": get_str_from_naivedate(end_date)
                    }
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

        let response_body: Value = es_client.get_search_query(&query, index_name).await?;

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
        let es_client: EsRepositoryPub = get_elastic_conn()?;

        let doc_id: &String = doc.id();

        es_client.delete_query(doc_id, index_name).await?;

        Ok(())
    }
}
