use crate::common::*;

use crate::service::es_service::*;

use crate::dtos::dto::*;


/*

*/
// pub async fn total_cost_specific_period(start_date: &str, end_date: &str, es_client: &Arc<EsHelper>, index_name: &str) -> Result<(i32, Vec<ConsumeInfo>), anyhow::Error> {

//     let query = json!({
//         "size": 10000,
//         "query": {
//             "range": {
//                 "@timestamp": {
//                     "gte": start_date,
//                     "lte": end_date
//                 }
//             }
//         },
//         "aggs": {
//             "total_prodt_money": {
//                 "sum": {
//                     "field": "prodt_money"
//                 }
//             }
//         },
//         "sort": {
//             "@timestamp": { "order": "asc" }
//         }
//     });

//     let es_cur_res = es_client.cluster_search_query(query, index_name).await?;
//     let total_cost = es_cur_res["aggregations"]["total_prodt_money"]["value"];

//     if let Some(prodt_infos) = es_cur_res["hits"]["hits"].as_array() {

//         for elem in prodt_infos {
            
//         }

//     }

//     println!("{:?}",es_cur_res);


//     Ok(12)
// }


/*

*/
pub async fn classification_consumption_type(es_client: &Arc<EsHelper>, index_name: &str) -> Result<Vec<ProdtTypeInfo>, anyhow::Error> {

    let query = json!({
        "size": 0,  
        "aggs": {
          "unique_keyword_types": {
            "terms": {
              "field": "keyword_type",
              "size": 100  
            }
          }
        }
    });

    let res = es_client.cluster_search_query(query, index_name).await?;
    let mut keyword_type_vec: Vec<ProdtTypeInfo> = Vec::new();

    if let Some(keyword_types) = res["aggregations"]["unique_keyword_types"]["buckets"].as_array() {
        
        for keyword_type in keyword_types {
            
            let k_type = match keyword_type.get("key").and_then(Value::as_str) {
                Some(k_type) => k_type,
                None => continue
            };

            let inner_query = json!({
                "query": {
                    "term": {
                        "keyword_type": {   
                            "value": k_type
                            }
                        }
                    },
                "size" : 1000
            });

            let inner_res = es_client.cluster_search_query(inner_query, index_name).await?;
            let mut keyword_vec: Vec<ProdtDetailInfo> = Vec::new();

            if let Some(keywords) = inner_res["hits"]["hits"].as_array() {
                for key_word in keywords {
                    if let Some(keyword_src) = key_word.get("_source") {
                        let k_word = keyword_src.get("keyword").and_then(Value::as_str);
                        let bias_value = keyword_src.get("bias_value").and_then(Value::as_i64).map(|v| v as i32);

                        info!("{:?} // {:?}", k_word, bias_value);

                        match (k_word, bias_value) {
                            (Some(word), Some(bias)) => {
                                let prodt_detail = ProdtDetailInfo::new(word.to_string(), bias);
                                keyword_vec.push(prodt_detail);
                            },
                            _ => {
                                error!("Error: Missing or invalid 'keyword' or 'bias_value'.");
                                continue;
                            }
                        }
                    }
                }
            }
            
            // let keyword_type_obj = ProdtTypeInfo::new(k_type.to_string(), keyword_vec);
            // keyword_type_vec.push(keyword_type_obj);
        }

    }
    
    Ok(keyword_type_vec)
}