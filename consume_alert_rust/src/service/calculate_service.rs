use crate::common::*;

use crate::service::es_service::*;
use crate::service::graph_api_service::*;

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

    let mut type_scores: HashMap<String, i32> = HashMap::new();
    
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
            type_scores.insert(keyword_type.to_string(), total_bias);
        } 
    }
    
    let mut confirm_type = String::from("");
    let mut max_score = 0;

    for (key, value) in &type_scores {
        
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



/*
    Function that returns consumption type information (Graphs and detailed consumption details)
*/
pub async fn get_consume_type_graph(total_cost: f64, start_dt: &str, end_dt: &str, consume_list: &Vec<ConsumeInfo>) -> Result<(Vec<ConsumeTypeInfo>, String), anyhow::Error> {

    let mut type_scores: HashMap<String, i32> = HashMap::new();
    
    for consume_info in consume_list {
        
        let prodt_money = consume_info.prodt_money;
        let prodt_type = (&consume_info.prodt_type).to_string();

        type_scores.entry(prodt_type)
            .and_modify(|e| *e += prodt_money)
            .or_insert(prodt_money);
    } 
    
    let mut consume_type_list: Vec<ConsumeTypeInfo> = Vec::new();
    
    for (key, value) in &type_scores {
        
        let prodt_type = key.to_string();
        let prodt_cost = *value;
        let prodt_per = (prodt_cost as f64 / total_cost) * 100.0; 
        let prodt_per_rounded = (prodt_per * 100.0).round() / 100.0; // Round to the second decimal place

        let consume_type_info = ConsumeTypeInfo::new(prodt_type, prodt_cost, prodt_per_rounded);
        consume_type_list.push(consume_type_info);
    }  
    
    consume_type_list.sort_by(|a, b| b.prodt_cost.cmp(&a.prodt_cost));
    
    let png_path = call_python_matplot_consume_type(&consume_type_list, start_dt, end_dt, total_cost).await?;
    
    Ok((consume_type_list, png_path))
}


/*

*/
pub async fn get_consume_detail_graph_double(python_graph_line_info_cur: ToPythonGraphLine, python_graph_line_info_pre: ToPythonGraphLine) -> Result<String, anyhow::Error> {

    let mut python_graph_line_vec: Vec<ToPythonGraphLine> = Vec::new();
    python_graph_line_vec.push(python_graph_line_info_cur);
    python_graph_line_vec.push(python_graph_line_info_pre);

    call_python_matplot_consume_detail(&python_graph_line_vec).await?;

    Ok(String::from("test"))
}


/*

*/
pub async fn get_consume_detail_graph_single(python_graph_line_info: &ToPythonGraphLine) -> Result<String, anyhow::Error> {

    let mut python_graph_line_vec: Vec<ToPythonGraphLine> = Vec::new();
    python_graph_line_vec.push(python_graph_line_info.clone());

    call_python_matplot_consume_detail(&python_graph_line_vec).await?;

    Ok(String::from("test"))
}