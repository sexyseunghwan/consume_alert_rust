use crate::common::*;

use crate::repository::es_repository::*;

use crate::models::score_manager::*;
use crate::models::consume_index_prodt_type::*;

#[async_trait]
pub trait ElasticQueryService {
    async fn get_consume_type_judgement(&self, prodt_name: &str) -> Result<String, anyhow::Error>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct ElasticQueryServicePub;

#[async_trait]
impl ElasticQueryService for ElasticQueryServicePub {

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

        let response_body: Value = es_client.get_search_query(&es_query, CONSUME_TYPE).await?;
        let hits: &Value = &response_body["hits"]["hits"];

        let results: Vec<ConsumingIndexProdType> = hits
            .as_array()
            .ok_or_else(|| anyhow!("[Error][get_consume_type_judgement()] error"))?
            .iter()
            .map(|hit| {
                hit.get("_source")
                    .ok_or_else(|| {
                        anyhow!("[Error][get_consume_type_judgement()] Missing '_source' field")
                    })
                    .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if results.len() == 0 {
            return Ok(String::from("etc"));
        } else {
            let mut manager: ScoreManager<ConsumingIndexProdtType> =
                ScoreManager::<ConsumingIndexProdtType>::new();

            for consume_type in results {
                let keyword: &String = consume_type.consume_keyword();

                /* Use the 'levenshtein' algorithm to determine word match */
                let word_dist: usize = levenshtein(keyword, &prodt_name);
                let word_dist_i32: i32 = word_dist.try_into()?;
                manager.insert(word_dist_i32, consume_type);
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

}



