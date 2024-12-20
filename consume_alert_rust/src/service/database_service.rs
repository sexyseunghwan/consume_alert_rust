use crate::common::*;

//use crate::service::es_service::*;
use crate::service::graph_api_service::*;

use crate::utils_modules::time_utils::*;

//use crate::repository::es_multi_repository::*;
use crate::repository::es_repository::*;

use crate::model::ProdtTypeInfo::*;
use crate::model::ProdtDetailInfo::*;
use crate::model::ConsumeInfo::*;
use crate::model::TotalCostInfo::*;
use crate::model::ConsumeTypeInfo::*;
use crate::model::ToPythonGraphLine::*;
use crate::model::MealCheckIndex::*;
use crate::model::ConsumingIndexProdType::*;
use crate::model::ConsumeIndexProd::*;
use crate::model::ConsumeIndexProdNew::*;


#[async_trait]
pub trait DBService {
    //async fn get_classification_type(&self, index_name: &str) -> Result<Vec<String>, anyhow::Error>;
    //async fn get_classification_consumption_type(&self, index_name: &str) -> Result<HashMap<String, ConsumingIndexProdType>, anyhow::Error>;
    //async fn get_classification_consumption_type(&self, index_name: &str) -> Result<HashMap<String, ConsumingIndexProdType>, anyhow::Error>;
    async fn get_consume_detail_specific_period(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<(f64, Vec<ConsumeIndexProd>), anyhow::Error>;
    //async fn get_classification_consume_detail(&self, consume_details: &mut [ConsumeIndexProd]) -> Result<(), anyhow::Error>;
    //async fn get_consume_info_by_classification_type<'a>(&self, consume_type_vec: &'a Vec<ProdtTypeInfo>, consume_info: &'a mut ConsumeInfo) -> Result<(), anyhow::Error>;
    // async fn get_consume_type_graph(total_cost: f64, start_dt: NaiveDate, end_dt: NaiveDate, consume_list: &Vec<ConsumeInfo>) -> Result<(Vec<ConsumeTypeInfo>, String), anyhow::Error>;
    // async fn get_consume_detail_graph_double(python_graph_line_info_cur: &mut ToPythonGraphLine, python_graph_line_info_pre: &mut ToPythonGraphLine) -> Result<String, anyhow::Error>;
    // async fn get_consume_detail_graph_single(python_graph_line_info: &ToPythonGraphLine) -> Result<String, anyhow::Error>;
    async fn get_recent_mealtime_data_from_elastic(&self, query_size: i32) -> Result<Vec<MealCheckIndex>, anyhow::Error>;
    
    async fn post_model_to_es<T: Serialize + Send>(&self, index_name: &str, model: T) -> Result<(), anyhow::Error>;

    async fn get_recent_consume_info_order_by(&self, order_by_info: &str, size: i64) -> Result<Vec<(String, ConsumeIndexProdNew)>, anyhow::Error>;
    async fn delete_es_doc(&self, index_name: &str, doc_id: &str) -> Result<(), anyhow::Error>;
}   

#[derive(Debug, Getters, new)]
pub struct DBServicePub;


#[async_trait]
impl DBService for DBServicePub {
    
    
    #[doc = "Function to remove specific document from index"]
    /// # Arguments
    /// * `index_name`
    /// * `doc_id`
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn delete_es_doc(&self, index_name: &str, doc_id: &str) -> Result<(), anyhow::Error> {

        let es_client = get_elastic_conn()?;
        es_client.delete_query(doc_id, index_name).await?;

        info!("Delete success - index: {}, doc_id: {}", index_name, doc_id);

        Ok(())
    }
    

    #[doc = ""]
    /// # Arguments
    /// * `order_by_info`
    /// 
    /// # Returns
    /// * Result<(&str,ConsumeIndexProdNew), anyhow::Error>
    async fn get_recent_consume_info_order_by(&self, order_by_info: &str, size: i64) -> Result<Vec<(String, ConsumeIndexProdNew)>, anyhow::Error> {
        
        let es_client = get_elastic_conn()?;
        let mut res_vec: Vec<(String, ConsumeIndexProdNew)> = Vec::new();

        /* Get the most up-to-date data through 'order_by' information. */ 
        let query = json!({
            "sort": {
                order_by_info : "desc"
            },
            "size": size
        });
        
        /* The information and ID of the consumption data are obtained and returned. */ 
        let response_body = es_client.get_search_query(&query, CONSUME_DETAIL).await?;
        let hits = &response_body["hits"]["hits"];

        if let Some(hit_array) = hits.as_array() {
            for hit in hit_array {
                let doc_id = hit.get("_id")
                    .ok_or_else(|| anyhow!("[Error][get_recent_consume_info_order_by()] Missing 'doc_id' field"))?
                    .to_string();

                let doc_id_trim = doc_id.trim_matches('"');

                let consume_info: ConsumeIndexProdNew = hit.get("_source")
                    .ok_or_else(|| anyhow!("[Error][get_recent_consume_info_order_by()] Missing '_source' field"))
                    .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))?;
                
                res_vec.push((doc_id_trim.to_string(), consume_info));
            }
        }  
        
        Ok(res_vec)
    }
    
    
    // #[doc = "Function to get ProdtTypeInfo keyword_type information from Elasticsearch"]
    // async fn get_classification_type(&self, index_name: &str) -> Result<Vec<String>, anyhow::Error> {
        
    //     let query = json!({
    //         "size": 0,  
    //         "aggs": {
    //           "unique_keyword_types": {
    //             "terms": {
    //               "field": "keyword_type",
    //               "size": 1000  
    //             }
    //           }
    //         }
    //     });
        
    //     let es_client = get_elastic_conn(); 
    //     let query_res = es_client.get_search_query(&query, index_name).await?;
    //     let mut key_vec: Vec<String> = Vec::new();

    //     if let Some(keyword_types) = query_res["aggregations"]["unique_keyword_types"]["buckets"].as_array() {

    //         for keyword_type in keyword_types {
                
    //             let k_type = match keyword_type.get("key").and_then(Value::as_str) {
    //                 Some(k_type) => k_type,
    //                 None => continue
    //             };

    //             key_vec.push(k_type.to_string());
    //         }
    //     }
        
    //     Ok(key_vec)
    // }
    
    
    // 이것도 굳이 필요가 없는데
    // #[doc = "Function to get FULL ProdtTypeInfo keyword_type information from Elasticsearch"]
    // async fn get_classification_consumption_type(&self, index_name: &str) -> Result<HashMap<String, ConsumingIndexProdType>, anyhow::Error> {

    //     let query = json!({
    //         "size": 10000
    //     });  
        
    //     let mut consume_type_map: HashMap<String, ConsumingIndexProdType> = HashMap::new();
    //     let es_client = get_elastic_conn(); 
    //     let response_body = es_client.get_search_query(&query, index_name).await?;
    //     let hits = &response_body["hits"]["hits"];

    //     let results: Vec<ConsumingIndexProdType> = hits.as_array()
    //         .ok_or_else(|| anyhow!("[Error][get_recent_mealtime_data_from_elastic()] "))?
    //         .iter()
    //         .map(|hit| {
    //             hit.get("_source") 
    //                 .ok_or_else(|| anyhow!("[Error][get_recent_mealtime_data_from_elastic()] Missing '_source' field"))
    //                 .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
    //         })
    //         .collect::<Result<Vec<_>, _>>()?;
        
    //     for elem in &results {
    //         let keyword = elem.keyword();
    //         consume_type_map.insert(keyword.to_string(), elem.clone());
    //     }
        
    //     Ok(consume_type_map)
        
    //     //let key_vec = self.get_classification_type(index_name).await?;// 필요없을거 같은데...
        
    //     //let mut keyword_type_vec: Vec<ProdtTypeInfo> = Vec::new();
    //     // let mut keyword_type_hashmap: HashMap<String, String> = HashMap::new();
    //     // let es_client = get_elastic_conn(); 
        
    //     // for keyword_type in key_vec {

    //     //     let inner_query = json!({
    //     //         "query": {
    //     //             "term": {
    //     //                 "keyword_type": {   
    //     //                     "value": keyword_type
    //     //                     }
    //     //                 }
    //     //             },
    //     //         "size" : 1000
    //     //     });
            
    //     //     //let mut keyword_vec: Vec<ProdtDetailInfo> = Vec::new();
    //     //     let response_body = es_client.get_search_query(&inner_query, index_name).await?;
    //     //     let hits = &response_body["hits"]["hits"];
            
    //     //     let results: Vec<ConsumingIndexProdType> = hits.as_array()
    //     //         .ok_or_else(|| anyhow!("[Error][get_recent_mealtime_data_from_elastic()] "))?
    //     //         .iter()
    //     //         .map(|hit| {
    //     //             hit.get("_source") 
    //     //                 .ok_or_else(|| anyhow!("[Error][get_recent_mealtime_data_from_elastic()] Missing '_source' field"))
    //     //                 .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
    //     //         })
    //     //         .collect::<Result<Vec<_>, _>>()?;
            
    //     //     for elem in &results {
    //     //         keyword_type_hashmap.insert();
    //     //     }

    //         //keyword_type_hashmap.insert(keyword_type, results);
    
    //         // if let Some(keywords) = inner_res["hits"]["hits"].as_array() {
    //         //     for key_word in keywords {
    //         //         if let Some(keyword_src) = key_word.get("_source") {
    //         //             let k_word = keyword_src.get("keyword").and_then(Value::as_str);
    //         //             let bias_value = keyword_src.get("bias_value").and_then(Value::as_i64).map(|v| v as i32);

    //         //             match (k_word, bias_value) {
    //         //                 (Some(word), Some(bias)) => {
    //         //                     let prodt_detail = ProdtDetailInfo::new(word.to_string(), bias);
    //         //                     keyword_vec.push(prodt_detail);
    //         //                 },
    //         //                 _ => {
    //         //                     error!("[Parsing Error][get_classification_consumption_type()] Missing or invalid 'keyword' or 'bias_value'");
    //         //                     continue;
    //         //                 }
    //         //             }
    //         //         }
    //         //     }
    //         // }
            
    //         // let keyword_type_obj = ProdtTypeInfo::new(keyword_type, keyword_vec);
    //         // keyword_type_vec.push(keyword_type_obj);
    //     //}
        
    //     //Ok(keyword_type_hashmap)
    // }
    
    
    #[doc = "Functions that show the details of total consumption and consumption over a specific period of time"]
    /// # Arguments
    /// * `start_date`      
    /// * `end_date`        
    ///    
    /// 
    /// # Returns
    /// * Result<(f64, Vec<ConsumeIndexProd>), anyhow::Error> - (total cost, Vec<ConsumeIndexProd>)
    async fn get_consume_detail_specific_period(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<(f64, Vec<ConsumeIndexProd>), anyhow::Error> {
         
        //let start = std::time::Instant::now();

        let query = json!({
            "size": 10000,
            "query": {
                "range": {
                    "@timestamp": {
                        "gte": get_str_from_naivedate(start_date),
                        "lte": get_str_from_naivedate(end_date)
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
        
        let es_client = get_elastic_conn()?; 
        let response_body = es_client.get_search_query(&query, CONSUME_DETAIL).await?;
        let hits = &response_body["hits"]["hits"];
        
        /* Total money spent in a particular period of time */
        let total_cost = match &response_body["aggregations"]["total_prodt_money"]["value"].as_f64() {
            Some(total_cost) => *total_cost,
            None => return Err(anyhow!("[Error][total_cost_detail_specific_period()] 'total_cost' error"))
        };
        
        //let duration = start.elapsed(); // 경과 시간 계산
        
        let mut results: Vec<ConsumeIndexProd> = hits.as_array()
            .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] error"))?
            .iter()
            .map(|hit| {
                hit.get("_source") 
                    .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] Missing '_source' field"))
                    .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
            })
            .collect::<Result<Vec<_>, _>>()?; 
        
        
        //get_classification_consume_detail_v1(&mut results).await?;
        

        /* 어떤 소비타입인지 분류해주는 로직필요 */
        let total = results.len();
        let chunk_size = total / 2;
        
        let tasks: Vec<_> = (0..2).map(|i| {
            
            let start = i * chunk_size;
            let end = if i == 1 { total } else { start + chunk_size };
            let slice = results[start..end].to_vec();  
                      
            tokio::spawn(async move {
                /* 비동기 함수 내부에서 slice를 수정 */ 
                get_classification_consume_detail(slice).await
            })


        }).collect();
        
        /* Wait for the results of all tasks */  
        let mut results: Vec<ConsumeIndexProd> = Vec::with_capacity(total);

        for task in tasks {
            match task.await {
                Ok(Ok(part)) => results.extend(part), /* Task success */
                Ok(Err(e)) => {
                    /* Error in get_classification_consume_detail() */
                    return Err(anyhow!("[Error][get_classification_consume_detail()] {:?}", e));
                },
                Err(e) => {
                    /* Error in TASK */
                    return Err(anyhow!("[Error][get_consume_detail_specific_period()] {:?}", e));
                }
            }
        }

        Ok((total_cost, results)) 
    }
    
    
    // #[doc = "Functions that show the details of total consumption and consumption over a specific period of time"]
    // /// # Arguments
    // /// * `start_date`      
    // /// * `end_date`        
    // /// * `index_name`       
    // /// * `consume_type_vec`- Consumption Type Information Vector ex) Meals, cafes, etc...
    // /// 
    // /// # Returns
    // /// *  Result<TotalCostInfo, anyhow::Error>
    // async fn total_cost_detail_specific_period(&self, start_date: NaiveDate, end_date: NaiveDate, index_name: &str, consume_type_vec: &HashMap<String, Vec<ConsumingIndexProdType>>) -> Result<TotalCostInfo, anyhow::Error> {
        
    //     let query = json!({
    //         "size": 10000,
    //         "query": {
    //             "range": {
    //                 "@timestamp": {
    //                     "gte": get_str_from_naivedate(start_date),
    //                     "lte": get_str_from_naivedate(end_date)
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
        
    //     let mut consume_info_list:Vec<ConsumeInfo> = Vec::new();
    //     let mut empty_flag = false;
        
    //     let es_client = get_elastic_conn(); 
    //     let response_body = es_client.get_search_query(&query, index_name).await?;
    //     let hits = &response_body["hits"]["hits"];

    //     let total_cost = match &response_body["aggregations"]["total_prodt_money"]["value"].as_f64() {
    //         Some(total_cost) => *total_cost,
    //         None => return Err(anyhow!("[Error][total_cost_detail_specific_period()] 'total_cost' error"))
    //     };
        
    //     let mut results: Vec<ConsumeIndexProd> = hits.as_array()
    //         .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] "))?
    //         .iter()
    //         .map(|hit| {
    //             hit.get("_source") 
    //                 .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] Missing '_source' field"))
    //                 .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
    //         })
    //         .collect::<Result<Vec<_>, _>>()?; 
        
        
        
    //     // if let Some(prodt_infos) = es_cur_res["hits"]["hits"].as_array() {

    //     //     for elem in prodt_infos {

    //     //         if let Some(source) = elem.get("_source") {
                    
    //     //             let timestamp = match source.get("@timestamp").and_then(Value::as_str) {
    //     //                 Some(timestamp) => timestamp,
    //     //                 None => {
    //     //                     errork(anyhow!("[Error][total_cost_detail_specific_period()] '@timestamp' is empty!")).await;
    //     //                     continue
    //     //                 }
    //     //             };
                    
    //     //             let prodt_money = match source.get("prodt_money").and_then(Value::as_i64).map(|v| v as i32) {
    //     //                 Some(timestamp) => timestamp,
    //     //                 None => {
    //     //                     errork(anyhow!("[Error][total_cost_detail_specific_period()] 'prodt_money' is empty!")).await;
    //     //                     continue
    //     //                 }
    //     //             };
        
    //     //             let prodt_name = match source.get("prodt_name").and_then(Value::as_str) {
    //     //                 Some(timestamp) => timestamp,
    //     //                 None => {
    //     //                     errork(anyhow!("[Error][total_cost_detail_specific_period()] 'prodt_name' is empty!")).await;
    //     //                     continue
    //     //                 }
    //     //             };
                    
    //     //             let mut consume_info = ConsumeInfo::new(timestamp.to_string(), prodt_name.to_string(), prodt_money, String::from(""));
    //     //             self.get_consume_info_by_classification_type(consume_type_vec, &mut consume_info).await?;
                    
    //     //             consume_info_list.push(consume_info);
    //     //         }             
    //     //     }
    //     // }
        

    //     // if consume_info_list.len() == 0 {
    //     //     let cur_time = get_str_from_naive_datetime(get_current_kor_naive_datetime());
    //     //     let consume_info = ConsumeInfo::new(cur_time, String::from("empty"), 0, String::from("etc"));

    //     //     consume_info_list.push(consume_info);
    //     //     empty_flag = true;
    //     // }
        
    //     // let total_cost_info = TotalCostInfo::new(total_cost, consume_info_list, empty_flag, start_date, end_date);

    //     //Ok(total_cost_info)
    // }



    // #[doc = "function that classifies what category that consumption is and returns an 'ConsumeInfo' instance."]
    // async fn get_consume_info_by_classification_type<'a>(&self, consume_type_vec: &'a Vec<ProdtTypeInfo>, consume_info: &'a mut ConsumeInfo) -> Result<(), anyhow::Error> {

    //     let mut type_scores_map: HashMap<String, i32> = HashMap::new();
        
    //     let prodt_name_trim = consume_info.prodt_name().trim();

    //     for prodt_type_info in consume_type_vec {

    //         let keyword_type = prodt_type_info.keyword_type(); /* consumption classification. For example, pizza Hut is classified as a MEAL */
    //         let mut total_bias = 0;

    //         for prodt_detail in prodt_type_info.prodt_detail_list() {
                
    //             let keyword = prodt_detail.keyword();    /* Pizza hut(keyword) ⊂ Meal(keyword_type) */
    //             let bias_value = prodt_detail.bias_value(); /* Weight of the corresponding keyword */

    //             if prodt_name_trim.contains(keyword) {
    //                 total_bias += bias_value;
    //             }
    //         }
            
    //         if total_bias != 0 {
    //             type_scores_map.insert(keyword_type.to_string(), total_bias);
    //         } 
    //     }
        
    //     let mut confirm_keyword_type = String::from("");
    //     let mut max_score = 0;

    //     /* Categories are determined based on the larger max_score. */
    //     for (key, value) in &type_scores_map {
            
    //         let keyword_type = key.to_string();
    //         let keyword_score = *value;
            
    //         if keyword_score > max_score {
    //             max_score = keyword_score;
    //             confirm_keyword_type = keyword_type;
    //         }
    //     }
        
    //     if max_score == 0 {
    //         consume_info.prodt_type = String::from("etc"); /* "max_score = 0" means that the consumption details are NOT included in any category. */
    //     } else {
    //         consume_info.prodt_type = confirm_keyword_type;
    //     }
        
    //     Ok(())
    // }


    
    #[doc = "Function that determines the number of meals today"]
    async fn get_recent_mealtime_data_from_elastic(&self, query_size: i32) -> Result<Vec<MealCheckIndex>, anyhow::Error> {
        
        let es_client = get_elastic_conn()?;
        let current_date = get_current_kor_naivedate().to_string();
        
        let es_query = json!({
            "size": query_size,
            "query": {
                "range": {
                    "@timestamp": {
                        "gte": "2023-11-27",    //&current_date,
                        "lte": "2023-11-27"     //&current_date
                    }
                }
            },
            "sort": [
                { "@timestamp": { "order": "desc" } }
            ]
        });
        
        let response_body = es_client.get_search_query(&es_query, "meal_check_index").await?;
        let hits = &response_body["hits"]["hits"];
        
        let results: Vec<MealCheckIndex> = hits.as_array()
            .ok_or_else(|| anyhow!("[Error][get_recent_mealtime_data_from_elastic()] "))?
            .iter()
            .map(|hit| {
                hit.get("_source") 
                    .ok_or_else(|| anyhow!("[Error][get_recent_mealtime_data_from_elastic()] Missing '_source' field"))
                    .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(results)
    }


    
    #[doc = "Function that posts the Json model itself to an Elasticsearch specific index"]
    /// # Arguments
    /// * model - Model to POST with Elasticsearch
    /// 
    /// # Result
    /// * Result<(), anyhow::Error>
    async fn post_model_to_es<T: Serialize + Send>(&self, index_name: &str, model: T) -> Result<(), anyhow::Error> {
        
        let es_client = get_elastic_conn()?;

        let model_json = serde_json::to_value(model)?; 
        es_client.post_query(&model_json, index_name).await?;

        Ok(())
    }
     
}




// /*
//     function that classifies what category that consumption is and returns an "ConsumeInfo" instance.
// */
// pub async fn get_consume_info_by_classification_type<'a>(consume_type_vec: &'a Vec<ProdtTypeInfo>, consume_info: &'a mut ConsumeInfo) -> Result<(), anyhow::Error> {

//     let mut type_scores: HashMap<String, i32> = HashMap::new();
    
//     let prodt_name_trim = consume_info.prodt_name().trim(); // Remove both spaces

//     for prodt_type_info in consume_type_vec {

//         let keyword_type = prodt_type_info.keyword_type();
//         let mut total_bias = 0;

//         for prodt_detail in prodt_type_info.prodt_detail_list() {
            
//             let keyword = prodt_detail.keyword();
//             let bias_value = prodt_detail.bias_value();

//             if prodt_name_trim.contains(keyword) {
//                 total_bias += bias_value;
//             }
//         }
        
//         if total_bias != 0 {
//             type_scores.insert(keyword_type.to_string(), total_bias);
//         } 
//     }
    
//     let mut confirm_type = String::from("");
//     let mut max_score = 0;

//     for (key, value) in &type_scores {
        
//         let in_key = key.to_string();
//         let in_value = *value;
        
//         if in_value > max_score {
//             max_score = in_value;
//             confirm_type = in_key;
//         }
//     }

//     if max_score == 0 {
//         consume_info.prodt_type = String::from("etc");
//     } else {
//         consume_info.prodt_type = confirm_type;
//     }
    
//     Ok(())
// }



// /*
//     Function that returns consumption type information (Graphs and detailed consumption details)
// */
// pub async fn get_consume_type_graph(total_cost: f64, start_dt: NaiveDate, end_dt: NaiveDate, consume_list: &Vec<ConsumeInfo>) -> Result<(Vec<ConsumeTypeInfo>, String), anyhow::Error> {

//     let mut type_scores: HashMap<String, i32> = HashMap::new();
    
//     for consume_info in consume_list {
        
//         let prodt_money = consume_info.prodt_money;
//         let prodt_type = consume_info.prodt_type.to_string();
        
//         type_scores.entry(prodt_type)
//             .and_modify(|e| *e += prodt_money)
//             .or_insert(prodt_money);
//     } 
    
//     let mut consume_type_list: Vec<ConsumeTypeInfo> = Vec::new();
    
//     for (key, value) in &type_scores {
        
//         let prodt_type = key.to_string();
//         let prodt_cost = *value;

//         if prodt_cost == 0 { continue; }

//         let prodt_per = (prodt_cost as f64 / total_cost) * 100.0; 
//         let prodt_per_rounded = (prodt_per * 10.0).round() / 10.0; // Round to the second decimal place

//         let consume_type_info = ConsumeTypeInfo::new(prodt_type, prodt_cost, prodt_per_rounded);
//         consume_type_list.push(consume_type_info);
//     }  
    
//     consume_type_list.sort_by(|a, b| b.prodt_cost.cmp(&a.prodt_cost));
    
//     let png_path = call_python_matplot_consume_type(&consume_type_list, start_dt, end_dt, total_cost).await?;
    
//     Ok((consume_type_list, png_path))
// }



// /*
//     Function to graph detailed consumption information (two graphs)
// */
// pub async fn get_consume_detail_graph_double(python_graph_line_info_cur: &mut ToPythonGraphLine, python_graph_line_info_pre: &mut ToPythonGraphLine) -> Result<String, anyhow::Error> {
    
//     let python_graph_line_info_cur_len = python_graph_line_info_cur.get_consume_accumulate_list_len();
//     let python_graph_line_info_pre_len = python_graph_line_info_pre.get_consume_accumulate_list_len();
    
//     // Sorting Algorithm
//     match python_graph_line_info_cur_len.cmp(&python_graph_line_info_pre_len) {
//         Ordering::Greater => {
//             let last_elem_pre = python_graph_line_info_pre.consume_accumulate_list.get(python_graph_line_info_pre_len - 1)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_detail_graph_double()] The {}th data of 'python_graph_line_info_pre.consume_accumulate_list' vector does not exist.", python_graph_line_info_pre_len - 1))?;
            
//             python_graph_line_info_pre.add_to_consume_accumulate_list(*last_elem_pre);
//         },
//         Ordering::Less => {
//             let last_elem_cur = python_graph_line_info_cur.consume_accumulate_list.get(python_graph_line_info_cur_len - 1)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_detail_graph_double()] The {}th data of 'python_graph_line_info_cur.consume_accumulate_list' vector does not exist.", python_graph_line_info_cur_len - 1))?;
            
//             python_graph_line_info_cur.add_to_consume_accumulate_list(*last_elem_cur);
//         },
//         Ordering::Equal => { }
//     }
    
//     let python_graph_line_vec: Vec<ToPythonGraphLine> = vec![python_graph_line_info_cur.clone(), python_graph_line_info_pre.clone()];
//     let path = call_python_matplot_consume_detail(&python_graph_line_vec).await?;
    
//     Ok(path)
// }


// /*
//     Function to graph detailed consumption information (one graph)
// */
// pub async fn get_consume_detail_graph_single(python_graph_line_info: &ToPythonGraphLine) -> Result<String, anyhow::Error> {

//     let python_graph_line_vec: Vec<ToPythonGraphLine> = vec![python_graph_line_info.clone()];
//     let path = call_python_matplot_consume_detail(&python_graph_line_vec).await?;
    
//     Ok(path)
// }


// /*
//     Function that determines the number of meals today
// */
// pub async fn get_recent_mealtime_data_from_elastic<T: DeserializeOwned>(es_client: &Arc<EsHelper>, index_name: &str, col_name: &str, es_query: Value, default_val: T) -> Result<T, anyhow::Error> {
    
//     let es_res = es_client.cluster_search_query(es_query, index_name).await?;

//     if let Some(meal_info) = es_res["hits"]["hits"].as_array() {
//         for elem in meal_info {
//             if let Some(source) = elem.get("_source") {
//                 if let Some(value) = source.get(col_name) {
                    
//                     let get_data: T = from_value(value.clone())
//                         .map_err(|e| anyhow!("[Json Parsing Error][get_recent_mealtime_data_from_elastic()] Failed to parse '{}' : {:?}", col_name, e))?;

//                     return Ok(get_data);
//                 }
//             }
//         }
//     }
    
//     Ok(default_val)
// }


//async fn get_classification_consume_detail(consume_details: &mut tokio::sync::MutexGuard<'_, Vec<ConsumeIndexProd>>, start: usize, end: usize) -> Result<(), anyhow::Error> {

    // let es_client = get_elastic_conn(); 
    
    // for idx in start..end {

    //     let prodt_name = consume_details[idx].prodt_name();

    //     let query = json!({
    //         "query": {
    //             "match": {
    //                 "keyword": prodt_name
    //             }
    //         }
    //     });
        
    //     let response_body = es_client.get_search_query(&query, CONSUME_TYPE).await?;
    //     let hits = &response_body["hits"]["hits"];

    //     let results: Vec<ConsumingIndexProdType> = hits.as_array()
    //         .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] error"))?
    //         .iter()
    //         .map(|hit| {
    //             hit.get("_source") 
    //                 .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] Missing '_source' field"))
    //                 .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
    //         })
    //         .collect::<Result<Vec<_>, _>>()?;
        
    //     if results.len() == 0 {
    //         println!("@");
    //         consume_details[idx].prodt_type = Some(String::from("etc"));

    //     } else {
    //         println!("!");
    //         let mut score_pair: HashMap<String, usize> = HashMap::new();

    //         for consume_type in &results {
    //             let keyword_type = consume_type.keyword_type();
    //             let bias_value = levenshtein(keyword_type, prodt_name);
                
    //             let entry = score_pair.entry(keyword_type.to_string()).or_insert(0);
    //             *entry += bias_value;   
    //         }

    //         let top_score_consume_type = match score_pair.iter()
    //             .max_by_key(|entry| entry.1)
    //             .map(|(key, _)| key.to_string()) {
    //                 Some(top_score_consume_type) => top_score_consume_type,
    //                 None => {
    //                     error!("[Error][get_classification_consume_detail()] Data 'top_score_consume_type' cannot have a None value.");
    //                     continue;
    //                 }   
    //             };
            
    //         consume_details[idx].prodt_type = Some(top_score_consume_type);
    //     }
    // }

    // Ok(())


#[doc = ""]
async fn get_classification_consume_detail_v1(consume_details: &mut Vec<ConsumeIndexProd>) -> Result<(), anyhow::Error> {

    let es_client = get_elastic_conn()?; 
    
    for consume_detail in consume_details {

        let prodt_name = consume_detail.prodt_name();

        let query = json!({
            "query": {
                "match": {
                    "keyword": prodt_name
                }
            }
        });
        
        let response_body = es_client.get_search_query(&query, CONSUME_TYPE).await?;
        let hits = &response_body["hits"]["hits"];
        
        let results: Vec<ConsumingIndexProdType> = hits.as_array()
            .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] error"))?
            .iter()
            .map(|hit| {
                hit.get("_source") 
                    .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] Missing '_source' field"))
                    .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
            })
            .collect::<Result<Vec<_>, _>>()?;
        

        if results.len() == 0 {
            consume_detail.prodt_type = Some(String::from("etc"));
        } else {
            consume_detail.prodt_type_query_res = Some(results);
        }
    }

    Ok(())
}





#[doc = ""]
///
/// 
/// 
/// 
/// 
/// 
async fn get_classification_consume_detail(consume_details: Vec<ConsumeIndexProd>) -> Result<Vec<ConsumeIndexProd>, anyhow::Error> {

    let mut consume_details_inner: Vec<ConsumeIndexProd> = Vec::new();
    let es_client = get_elastic_conn()?; 
    
    for mut consume_detail in consume_details {
        
        let prodt_name = consume_detail.prodt_name();
        
        let query = json!({
            "query": {
                "match": {
                    "keyword": prodt_name
                }
            }
        });
        
        let response_body = es_client.get_search_query(&query, CONSUME_TYPE).await?;
        let hits = &response_body["hits"]["hits"];
        
        let results: Vec<ConsumingIndexProdType> = hits.as_array()
            .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] error"))?
            .iter()
            .map(|hit| {
                hit.get("_source") 
                    .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] Missing '_source' field"))
                    .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        
        if results.len() == 0 {
            
            consume_detail.prodt_type = Some(String::from("etc"));
            consume_details_inner.push(consume_detail);

        } else {

            let mut score_pair: HashMap<String, usize> = HashMap::new();
            
            for consume_type in &results {
                let keyword_type = consume_type.keyword_type();
                let word_dist = levenshtein(keyword_type, prodt_name);
                
                let entry = score_pair.entry(keyword_type.to_string()).or_insert(word_dist);
                *entry += word_dist;   
            }
            
            let top_score_consume_type = match score_pair.iter()
                .min_by_key(|entry| entry.1)
                .map(|(key, _)| key.to_string()) {
                    Some(top_score_consume_type) => top_score_consume_type,
                    None => {
                        error!("[Error][get_classification_consume_detail()] Data 'top_score_consume_type' cannot have a None value.");
                        continue;
                    }   
                };
            
            consume_detail.prodt_type = Some(top_score_consume_type);
            consume_details_inner.push(consume_detail);
        }
    }
    
    Ok(consume_details_inner)
}