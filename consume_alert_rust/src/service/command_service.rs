use crate::common::*;

use crate::service::es_service::*;

use crate::utils_modules::numeric_utils::*;
use crate::utils_modules::time_utils::*;

/*
    Function to send result message via Telegram Bot
*/
async fn tele_bot_send_msg(bot: &Bot, chat_id: ChatId, err_yn: bool, msg: &str, log_msg: &str) -> Result<(), anyhow::Error> {
    
    if err_yn {

        bot.send_message(chat_id, msg)
            .await
            .context("Failed to send command response")?;
        
        //error!("{:?}", log_msg);
        
    } else {
        
        bot.send_message(chat_id, msg)
            .await
            .context("Failed to send command response")?;
        
        info!("{:?}", log_msg);
    }
    
    Ok(())
}


/*
    command handler: Writes the expenditure details to the index in ElasticSearch. -> /c
*/
pub async fn command_consumption(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(":").map(|s| s.to_string()).collect();
    let mut consume_name = "";
    let mut consume_cash = "";

    if split_args_vec.len() != 2 {
        
        tele_bot_send_msg(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /c snack:15000","").await?;
        return Err(anyhow!(format!("ERROR in 'command_consumption()' function - input_data : {}", text)));
    } 

    if let Some(cons_name) = split_args_vec.get(0) {

        if let Some(price) = split_args_vec.get(1) {
            
            if !is_numeric(&price) {
                tele_bot_send_msg(bot, message.chat.id, true, "The second parameter must be numeric. \nEX) /c snack:15000", "").await?;
                return Err(anyhow!(format!("ERROR in 'command_consumption()' function - input_data : {}", text)));
            }

            consume_name = cons_name;
            consume_cash = price;
        }        

    } else {

        tele_bot_send_msg(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /c snack:15000", "").await?;
        return Err(anyhow!("ERROR in 'command_consumption()' function - input_data : {}", text));
    }
    
    let curr_time = get_current_korean_time_str("%Y-%m-%dT%H:%M:%S%.3fZ");
    
    let document = json!({
        "@timestamp": curr_time,
        "prodt_name": consume_name,
        "prodt_money": convert_numeric(consume_cash)
    });

    es_client.cluster_post_query(document, "consuming_index_prod_new").await?;
    
    Ok(())
}



/*
    command handler: Checks how much you have consumed during a month -> /cm
*/
pub async fn command_consumption_per_mon(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(" ").map(|s| s.to_string()).collect();

    let mut cur_date_start = String::from("");   
    let mut cur_date_end = String::from("");  
    let mut one_mon_ago_date_start = String::from("");    
    let mut one_mon_ago_date_end = String::from("");    

    // 1. Initialize current date, month prior date
    if split_args_vec.len() == 1 {
        
        // == [case1] /cm == 
        cur_date_start = get_current_korean_time_str("%Y-%m-01");
        cur_date_end = get_last_date_str(cur_date_start.as_str(), "%Y-%m-%d")?;
        one_mon_ago_date_start = get_one_month_ago_kr_str(cur_date_start.as_str(), "%Y-%m-01")?;
        one_mon_ago_date_end = get_last_date_str(one_mon_ago_date_start.as_str(), "%Y-%m-%d")?;
        
    } else if split_args_vec.len() == 2 {

        // == [case2] /cm 2024.01 ==
        if let Some(input_date) = split_args_vec.get(1) {

            let input_date_own = input_date.to_string();

            let date_format_yn = validate_date_format(input_date, r"^\d{4}\.\d{2}$")?;

            if !date_format_yn {
                tele_bot_send_msg(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX01) /cm 2023.07\nEX02) /cm","").await?;
                return Err(anyhow!(format!("ERROR in 'command_consumption_per_mon()' function - input_data : {}", text)));
            }
            
            cur_date_start = input_date_own + "-01";
            cur_date_end = get_last_date_str(cur_date_start.as_str(), "%Y-%m-%d")?;
            one_mon_ago_date_start = get_one_month_ago_kr_str(cur_date_start.as_str(), "%Y-%m-%d")?;
            one_mon_ago_date_end = get_last_date_str(one_mon_ago_date_start.as_str(), "%Y-%m-%d")?;

        } else {
            return Err(anyhow!(format!("ERROR in 'command_consumption_per_mon()' function - input_data : {}", text)));
        }
        
    } else {
        tele_bot_send_msg(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX01) /cm 2023.07.01\nEX02) /cm", "").await?;
        return Err(anyhow!(format!("The input parameter value of the 'command_consumption_per_mon()' function does not satisfy the specified date format. - input_val : {}", text)));
    }

    println!("??");

    // 2. It calculates the total amount of consumption.
    let cur_mon_total_cost = total_cost_specific_period(cur_date_start.as_str(), cur_date_end.as_str(), es_client, "consuming_index_prod_new").await?;
    let pre_mon_total_cost = total_cost_specific_period(one_mon_ago_date_start.as_str(), one_mon_ago_date_end.as_str(), es_client, "consuming_index_prod_new").await?;
    
    Ok(())
}



/*

*/
async fn total_cost_specific_period(start_date: &str, end_date: &str, es_client: &Arc<EsHelper>, index_name: &str) -> Result<i32, anyhow::Error> {

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
        }
    });


    let es_cur_res = es_client.cluster_search_query(query, index_name).await?;

    println!("{:?}",es_cur_res);

    Ok(12)
}