use crate::common::*;

use crate::dtos::dto::*;
use crate::service::es_service::*;
use crate::service::command_service::*;
use crate::service::calculate_service::*;
use crate::service::graph_api_service::*;



/*
    ======================================================
    ============= TEST Controller =============
    ======================================================
*/
pub async fn test_controller() {

    // Select compilation environment
    dotenv().ok();
    
    let es_host: Vec<String> = env::var("ES_DB_URL").expect("'ES_DB_URL' must be set").split(",").map(|s| s.to_string()).collect();
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
    
    let consume_type_vec = get_classification_consumption_type(&arc_es_client, "consuming_index_prod_type").await.unwrap();
    let (total_cost, consume_list) = total_cost_detail_specific_period("2024-06-01", "2024-06-15", &arc_es_client, "consuming_index_prod_new", &consume_type_vec).await.unwrap();
    
    let (consume_type_list, png_path) = get_consume_type_graph(total_cost, "2024-06-01", "2024-06-15", consume_list).await.unwrap();
    
    // 텔래그램으로 전송
    
    
    

    // for elem in res {
    //     println!("keyword_type: {:?}", elem.keyword_type);

    //     for in_elem in elem.prodt_detail_list {
    //         println!("keyword: {:?}", in_elem.keyword);
    //         println!("bias_value: {:?}", in_elem.bias_value);
    //     }

    //     println!("============================");
    // }

    // let mut test_consume_obj = ConsumeInfo::new(String::from(""), String::from("호야 초밥"), 12000, String::from(""));

    // let result = get_consume_info_by_classification_type(consume_type_vec, &mut test_consume_obj).await.unwrap();
    
    // println!("{:?}", result);

    // let query = json!({
    //     "size": 10000,
    //     "query": {
    //         "range": {
    //             "@timestamp": {
    //                 "gte": "2024-06-01",
    //                 "lte": "2024-07-01"
    //             }
    //         }
    //     },
    //     "aggs": {
    //         "total_prodt_money": {
    //             "sum": {
    //                 "field": "prodt_money"
    //             }
    //         }
    //     },
    //     "sort": {
    //         "@timestamp": { "order": "asc" }
    //     }
    // });


    // let es_cur_res = es_client.cluster_search_query(query, "consuming_index_prod_new").await.unwrap();

    // // total cost
    // println!("{:?}", es_cur_res["aggregations"]["total_prodt_money"]["value"]);
    // //println!("{:?}", es_cur_res["hits"]["hits"]);    

    // // for elem in es_cur_res["hits"]["hits"].as_array() {
    // //     println!("{:?}", elem);
    // //     println!("~~~~~~~~~~~~~~~~~~~~");
    // // }

    // // consume infos
    // if let Some(vec) = es_cur_res["hits"]["hits"].as_array() { 

    //     for elem in vec {
            
    //         println!("{:?}", elem);
    //     }
    //     println!("&&&&&&&&&&&&");

    // }


}



/*
    ======================================================
    ============= Telegram Bot Controller =============
    ======================================================
*/
pub async fn main_controller() {

    // Select compilation environment
    dotenv().ok();
    
    let es_host: Vec<String> = env::var("ES_DB_URL").expect("'ES_DB_URL' must be set").split(",").map(|s| s.to_string()).collect();
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
    
    // Telegram Bot - Read bot information from the ".env" file.
    let bot = Bot::from_env();

    // It wraps the Elasticsearch connection object with "Arc" for secure use in multiple threads.
    let arc_es_client: Arc<EsHelper> = Arc::new(es_client);

    //The ability to handle each command.
    teloxide::repl(bot, move |message: Message, bot: Bot| {

        let arc_es_client_clone = arc_es_client.clone();

        async move {
            match handle_command(&message, &bot, &arc_es_client_clone).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Error handling message: {:?}", e);
                }
            };
            respond(())
        }
    })
    .await;
    
}


/*
    Functions that handle each command
*/
async fn handle_command(message: &Message, bot: &Bot, arc_es_client_clone: &Arc<EsHelper>) -> Result<(), anyhow::Error> {
    
    if let Some(text) = message.text() {
        if text.starts_with("/c ") {
            command_consumption(message, text, bot, arc_es_client_clone).await?;
        } 
        else if text.starts_with("/cm") {
            command_consumption_per_mon(message, text, bot, arc_es_client_clone).await?;
        }
        else {
            bot.send_message(message.chat.id, "Hello! Use /c <args> to interact.")
                .await
                .context("Failed to send default interaction message")?;
        }
    }
    Ok(())
}