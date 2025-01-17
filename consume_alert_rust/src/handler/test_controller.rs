// // use core::time;

// // use crate::common::*;
// // use crate::service::es_service::*;
// // use crate::dtos::dto::*;
// // use crate::service::command_service::*;
// // use crate::service::calculate_service::*;
// // use crate::service::graph_api_service::*;

// // use crate::utils_modules::time_utils::*;

// /*
//     ======================================================
//     ============= TEST Controller =============
//     ======================================================
// */
// pub async fn test_controller() {

// //     // Select compilation environment
// //     dotenv().ok();

// //     let es_host: Vec<String> = env::var("ES_DB_URL").expect("'ES_DB_URL' must be set").split(',').map(|s| s.to_string()).collect();
// //     let es_id = env::var("ES_ID").expect("'ES_ID' must be set");
// //     let es_pw = env::var("ES_PW").expect("'ES_PW' must be set");

// //     // Elasticsearch connection
// //     let es_client: EsHelper = match EsHelper::new(es_host, &es_id, &es_pw) {
// //         Ok(es_client) => es_client,
// //         Err(err) => {
// //             error!("Failed to create mysql client: {:?}", err);
// //             panic!("Failed to create mysql client: {:?}", err);
// //         }
// //     };

// //     let arc_es_client: Arc<EsHelper> = Arc::new(es_client);

// //     // Telegram Bot - Read bot information from the ".env" file.
// //     let bot = Bot::from_env();

// //     let string_form = String::from("삼성5221승인 신*환
// // 3,300원 일시불
// // 09/01 10:25 (주)코리아세븐송
// // 누적2,926,302원

// // 채널 추가하고 이 채널의 마케팅 메시지 등을 카카오톡으로 받기");

// //     let string_form2 = String::from("[Web발신]
// // NH카드3*3*승인
// // 신*환
// // 11,600원 일시불
// // 09/07 10:02
// // 새희망약국
// // 총누적104,635원");

// //     let re = Regex::new(r"\[.*?\]\n?").unwrap();
// //     let replcae_string = re.replace_all(&string_form, "").to_string();

// //     //["삼성5221승인 신*환", "3,300원 일시불", "09/01 10:25 (주)코리아세븐송", "누적2,926,302원", "", "채널 추가하고 이 채널의 마케팅 메시지 등을 카카오톡으로 받기"]
// //     // ["NH카드3*3*승인 ", "신*환 ", "11,600원 일시불 ", "09/07 10:02 ", "새희망약국 ", "총누적104,635원"]
// //     // println!("{:?}", replcae_string);
// //     let split_args_vec: Vec<String> = replcae_string.split('\n').map(|s| s.to_string()).collect();

// //     println!("{:?}", split_args_vec);

// //     let card_comp = split_args_vec.get(0).unwrap();

// //     if card_comp.contains("NH") {
// //         println!("농협");

// //         let consume_price_vec: Vec<String> = split_args_vec
// //             .get(2)
// //             .unwrap()
// //             .replace(",", "")
// //             .replace("원", "")
// //             .split(" ")
// //             .map(|s| s.to_string()).collect();

// //         let consume_price = consume_price_vec.get(0).unwrap().parse::<i32>().unwrap();

// //         let consume_time_vec: Vec<String> = split_args_vec
// //             .get(3)
// //             .unwrap()
// //             .split(" ")
// //             .map(|s| s.to_string()).collect();

// //         let date_part: Vec<u32> = consume_time_vec.get(0).unwrap().split("/").map(|s| s.parse::<u32>().unwrap()).collect();
// //         let time_part: Vec<u32> = consume_time_vec.get(1).unwrap().split(":").map(|s| s.parse::<u32>().unwrap()).collect();

// //         let mon = date_part.get(0).unwrap();
// //         let day = date_part.get(1).unwrap();
// //         let hour = time_part.get(0).unwrap();
// //         let min = time_part.get(1).unwrap();

// //         let date = get_this_year_date_time(*mon, *day, *hour, *min).unwrap();

// //         let consume_name = split_args_vec
// //             .get(4)
// //             .unwrap()
// //             .trim();

// //         println!("consume_price: {:?}", consume_price);
// //         println!("date: {:?}", date);
// //         println!("consume_name: {:?}", consume_name);

// //     } else if card_comp.contains("삼성") {
// //         println!("삼성");

// //         let consume_price_vec: Vec<String> = split_args_vec
// //             .get(1)
// //             .unwrap()
// //             .replace(",", "")
// //             .replace("원", "")
// //             .split(" ")
// //             .map(|s| s.to_string()).collect();

// //         let consume_price = consume_price_vec.get(0).unwrap().parse::<i32>().unwrap();
// //         println!("{:?}", consume_price);

// //         let consume_time_name_vec: Vec<String> = split_args_vec
// //             .get(2)
// //             .unwrap()
// //             .split(" ")
// //             .map(|s| s.to_string()).collect();

// //         let date_part: Vec<u32> = consume_time_name_vec.get(0).unwrap().split("/").map(|s| s.parse::<u32>().unwrap()).collect();
// //         let time_part: Vec<u32> = consume_time_name_vec.get(1).unwrap().split(":").map(|s| s.parse::<u32>().unwrap()).collect();
// //         let consume_name = consume_time_name_vec.get(2).unwrap().trim();

// //         let mon = date_part.get(0).unwrap();
// //         let day = date_part.get(1).unwrap();
// //         let hour = time_part.get(0).unwrap();
// //         let min = time_part.get(1).unwrap();

// //         let date = get_this_year_date_time(*mon, *day, *hour, *min).unwrap();

// //         println!("{:?}", date);
// //         println!("{:?}", consume_name);

// //     } else {
// //         println!("아무것도 아님");
// //     }

//     // //get_recent_mealtime_data_from_elastic(&arc_es_client, "meal_check_index", "laststamp").await.unwrap();

//     // let start_dt = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
//     // let end_dt = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
//     // let pre_start_dt = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
//     // let pre_end_dt = NaiveDate::from_ymd_opt(2024, 5, 15).unwrap();

//     // let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(&arc_es_client, "consuming_index_prod_type").await.unwrap();
//     // /let (total_cost, consume_list) = total_cost_detail_specific_period(start_dt, end_dt, &arc_es_client, "consuming_index_prod_new", &consume_type_vec).await.unwrap();
//     // let (total_cost_pre, consume_list_pre) = total_cost_detail_specific_period(pre_start_dt, pre_end_dt, &arc_es_client, "consuming_index_prod_new", &consume_type_vec).await.unwrap();

//     // let python_graph_line_info_cur = ToPythonGraphLine::new("cur", &get_str_from_naivedate(start_dt), &get_str_from_naivedate(end_dt), total_cost, consume_list).unwrap();
//     // let python_graph_line_info_pre = ToPythonGraphLine::new("pre", &get_str_from_naivedate(pre_start_dt), &get_str_from_naivedate(pre_end_dt), total_cost_pre, consume_list_pre).unwrap();

//     // println!("python_graph_line_info_cur = {:?}", python_graph_line_info_cur);
//     // println!("python_graph_line_info_pre = {:?}", python_graph_line_info_pre);

//     // get_consume_detail_graph_double(python_graph_line_info_cur, python_graph_line_info_pre).await.unwrap();

//     // ======================================================================================================================================================

//     //let (consume_type_list, png_path) = get_consume_type_graph(total_cost, "2024-06-01", "2024-06-15", consume_list).await.unwrap();

//     // 텔래그램으로 전송
//     //println!("{:?}", consume_type_vec);

//     // for elem in consume_list {
//     //     println!("{:?}", elem);
//     // }

//     // let total_cost_i32 = total_cost as i32;
//     // let cnt = 10;
//     // let consume_list_len = consume_list.len();
//     // let mut loop_cnt: usize = 0;
//     // let consume_q = consume_list_len / cnt;
//     // let consume_r = consume_list_len % cnt;

//     // if consume_r != 0 { loop_cnt += consume_q + 1 }
//     // else { loop_cnt = consume_q }

//     // if consume_list_len == 0 {
//     //     // 소비내역 없는 경우
//     // }

//     // for idx in 0..loop_cnt {

//     //     let mut send_text = String::new();
//     //     let end_idx = cmp::min(consume_list_len, (idx+1)*cnt);

//     //     if idx == 0 {
//     //         send_text.push_str(&format!("The money you spent from [{} ~ {}] is [ {} won ] \n=========[DETAIL]=========\n", start_dt, end_dt, total_cost_i32.to_formatted_string(&Locale::ko)));
//     //     }

//     //     for inner_idx in (cnt*idx)..end_idx {
//     //         send_text.push_str("---------------------------------\n");
//     //         send_text.push_str(&format!("name : {}\n", consume_list[inner_idx].prodt_name()));
//     //         send_text.push_str(&format!("date : {}\n", consume_list[inner_idx].timestamp()));
//     //         send_text.push_str(&format!("cost : {}\n", consume_list[inner_idx].prodt_money().to_formatted_string(&Locale::ko)));
//     //     }

//     // }

// }
