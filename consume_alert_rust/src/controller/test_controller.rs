
use crate::common::*;
//use crate::dtos::dto::*;
use crate::service::es_service::*;
//use crate::service::command_service::*;
//use crate::service::calculate_service::*;
//use crate::service::graph_api_service::*;

//use crate::utils_modules::time_utils::*;


/*
    ======================================================
    ============= TEST Controller =============
    ======================================================
*/
pub async fn test_controller() {

    // Select compilation environment
    dotenv().ok();
    
    let es_host: Vec<String> = env::var("ES_DB_URL").expect("'ES_DB_URL' must be set").split(',').map(|s| s.to_string()).collect();
    let es_id = env::var("ES_ID").expect("'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("'ES_PW' must be set");

    // Elasticsearch connection
    let es_client: EsHelper = match EsHelper::new(es_host, &es_id, &es_pw) {
        Ok(es_client) => es_client,
        Err(err) => {
            error!("Failed to create mysql client: {:?}", err);
            panic!("Failed to create mysql client: {:?}", err);
        }
    };

    let arc_es_client: Arc<EsHelper> = Arc::new(es_client);

    //get_recent_mealtime_data_from_elastic(&arc_es_client, "meal_check_index", "laststamp").await.unwrap();
    
    
    
    // let start_dt = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    // let end_dt = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    // let pre_start_dt = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
    // let pre_end_dt = NaiveDate::from_ymd_opt(2024, 5, 15).unwrap();

    // let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(&arc_es_client, "consuming_index_prod_type").await.unwrap();
    // let (total_cost, consume_list) = total_cost_detail_specific_period(start_dt, end_dt, &arc_es_client, "consuming_index_prod_new", &consume_type_vec).await.unwrap();
    // let (total_cost_pre, consume_list_pre) = total_cost_detail_specific_period(pre_start_dt, pre_end_dt, &arc_es_client, "consuming_index_prod_new", &consume_type_vec).await.unwrap();
    
    // let python_graph_line_info_cur = ToPythonGraphLine::new("cur", &get_str_from_naivedate(start_dt), &get_str_from_naivedate(end_dt), total_cost, consume_list).unwrap();
    // let python_graph_line_info_pre = ToPythonGraphLine::new("pre", &get_str_from_naivedate(pre_start_dt), &get_str_from_naivedate(pre_end_dt), total_cost_pre, consume_list_pre).unwrap();
    
    // get_consume_detail_graph_double(python_graph_line_info_cur, python_graph_line_info_pre).await.unwrap();
    

    // ======================================================================================================================================================
    

    //let (consume_type_list, png_path) = get_consume_type_graph(total_cost, "2024-06-01", "2024-06-15", consume_list).await.unwrap();
    
    // 텔래그램으로 전송
    //println!("{:?}", consume_type_vec);
    
    // for elem in consume_list {
    //     println!("{:?}", elem);
    // }   

    // let total_cost_i32 = total_cost as i32;
    // let cnt = 10;
    // let consume_list_len = consume_list.len();
    // let mut loop_cnt: usize = 0;
    // let consume_q = consume_list_len / cnt;
    // let consume_r = consume_list_len % cnt;

    // if consume_r != 0 { loop_cnt += consume_q + 1 }
    // else { loop_cnt = consume_q }
    
    // if consume_list_len == 0 {
    //     // 소비내역 없는 경우
    // }
    
    // for idx in 0..loop_cnt {

    //     let mut send_text = String::new();
    //     let end_idx = cmp::min(consume_list_len, (idx+1)*cnt);

    //     if idx == 0 {
    //         send_text.push_str(&format!("The money you spent from [{} ~ {}] is [ {} won ] \n=========[DETAIL]=========\n", start_dt, end_dt, total_cost_i32.to_formatted_string(&Locale::ko)));
    //     } 

    //     for inner_idx in (cnt*idx)..end_idx {
    //         send_text.push_str("---------------------------------\n");
    //         send_text.push_str(&format!("name : {}\n", consume_list[inner_idx].prodt_name()));
    //         send_text.push_str(&format!("date : {}\n", consume_list[inner_idx].timestamp()));
    //         send_text.push_str(&format!("cost : {}\n", consume_list[inner_idx].prodt_money().to_formatted_string(&Locale::ko)));
    //     }


        
    // }
     
    



}