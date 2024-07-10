use crate::common::*;

use crate::service::es_service::*;

use crate::dtos::dto::*;




/*
    Function to get consumption classification information from Elasticsearch
*/
pub async fn get_classification_consumption_type(es_client: &Arc<EsHelper>, index_name: &str) -> Result<Vec<ProdtTypeInfo>, anyhow::Error> {

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
            
            let keyword_type_obj = ProdtTypeInfo::new(k_type.to_string(), keyword_vec);
            keyword_type_vec.push(keyword_type_obj);
        }
    }
    
    Ok(keyword_type_vec)
}



/*
    Functions that show the details of total consumption and consumption over a specific period of time
*/
pub async fn total_cost_detail_specific_period(start_date: &str, end_date: &str, es_client: &Arc<EsHelper>, index_name: &str, consume_type_vec: &Vec<ProdtTypeInfo>) -> Result<(f64, Vec<ConsumeInfo>), anyhow::Error> {

    let query = json!({
        "size": 10000,
        "query": {
            "range": {
                "@timestamp": {
                    "gte": start_date,
                    "lte": end_date
                }
            }
        },
        "aggs": {
            "total_prodt_money": {
                "sum": {
                    "field": "prodt_money"
                }
            }
        },
        "sort": {
            "@timestamp": { "order": "asc" }
        }
    });
    
    let mut consume_info_list:Vec<ConsumeInfo> = Vec::new();

    let es_cur_res = es_client.cluster_search_query(query, index_name).await?;
    let total_cost = match &es_cur_res["aggregations"]["total_prodt_money"]["value"].as_f64() {
        Some(total_cost) => *total_cost,
        None => return Err(anyhow!(format!("ERROR in 'total_cost_specific_period()'")))
    };

    if let Some(prodt_infos) = es_cur_res["hits"]["hits"].as_array() {

        for elem in prodt_infos {

            if let Some(source) = elem.get("_source") {
                
                let timestamp = match source.get("@timestamp").and_then(Value::as_str) {
                    Some(timestamp) => timestamp,
                    None => {
                        error!("'@timestamp' is empty!");
                        continue
                    }
                };
    
                let prodt_money = match source.get("prodt_money").and_then(Value::as_i64).map(|v| v as i32) {
                    Some(timestamp) => timestamp,
                    None => {
                        error!("'prodt_money' is empty!");
                        continue
                    }
                };
    
                let prodt_name = match source.get("prodt_name").and_then(Value::as_str) {
                    Some(timestamp) => timestamp,
                    None => {
                        error!("'prodt_name' is empty!");
                        continue
                    }
                };
                
                let mut consume_info = ConsumeInfo::new(timestamp.to_string(), prodt_name.to_string(), prodt_money, String::from(""));
                get_consume_info_by_classification_type(consume_type_vec, &mut consume_info).await?;
                
                consume_info_list.push(consume_info);
            }             
        }
    } 
    
    Ok((total_cost, consume_info_list))
}




/*
    function that classifies what category that consumption is and returns an "ConsumeInfo" instance.
*/
pub async fn get_consume_info_by_classification_type<'a>(consume_type_vec: &'a Vec<ProdtTypeInfo>, consume_info: &'a mut ConsumeInfo) -> Result<(), anyhow::Error> {

    let mut type_cores: HashMap<String, i32> = HashMap::new();
    
    let prodt_name_trim = consume_info.prodt_name().trim(); // Remove both spaces

    for prodt_type_info in consume_type_vec {

        let keyword_type = prodt_type_info.keyword_type();
        let mut total_bias = 0;

        for prodt_detail in prodt_type_info.prodt_detail_list() {
            
            let keyword = prodt_detail.keyword();
            let bias_value = prodt_detail.bias_value();

            if prodt_name_trim.contains(keyword) {
                total_bias += bias_value;
            }
        }
        
        if total_bias != 0 {
            type_cores.insert(keyword_type.to_string(), total_bias);
        } 
    }
    
    let mut confirm_type = String::from("");
    let mut max_score = 0;

    for (key, value) in &type_cores {
        
        let in_key = key.to_string();
        let in_value = *value;
        
        if in_value > max_score {
            max_score = in_value;
            confirm_type = in_key;
        }
    }

    if max_score == 0 {
        consume_info.prodt_type = String::from("etc");
    } else {
        consume_info.prodt_type = confirm_type;
    }
    
    Ok(())
}