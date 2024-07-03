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
        
        error!("{:?}", log_msg);
        
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
pub async fn command_consumption(message: &Message, text: &str, bot: &Bot, es_client: &EsHelper) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(":").map(|s| s.to_string()).collect();
    let mut consume_name = "";
    let mut consume_cash = "";

    if split_args_vec.len() != 2 {
        
        tele_bot_send_msg(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /c snack:15000", 
            format!("There are not two parameters input to the 'command_consumption_per_term()' function - input_data : {}", args).as_str()).await?;

        return Ok(());
    } 

    if let Some(cons_name) = split_args_vec.get(0) {

        if let Some(price) = split_args_vec.get(1) {
            
            if !is_numeric(&price) {
                tele_bot_send_msg(bot, message.chat.id, true, "The second parameter must be numeric. \nEX) /c snack:15000", 
                format!("There are not two parameters input to the 'command_consumption_per_term()' function - input_data : {}", args).as_str()).await?;
                
                return Ok(());
            }

            consume_name = cons_name;
            consume_cash = price;
        }        

    } else {

        tele_bot_send_msg(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /c snack:15000", 
            format!("There are not two parameters input to the 'command_consumption_per_term()' function - input_data : {}", args).as_str()).await?;

        return Ok(());
    }
    
    let curr_time = get_current_korean_time_str("%Y-%m-%dT%H:%M:%S%.3fZ");
    
    let document = json!({
        "@timestamp": curr_time,
        "prodt_name": consume_name,
        "prodt_money": convert_numeric(consume_cash)
    });

    let index_name = "consuming_index_prod_new";

    es_client.cluster_post_query(document, index_name).await?;
    
    Ok(())
}



/*
    command handler: Checks how much you have consumed during a month -> /cm
*/
pub async fn command_consumption_per_mon(message: &Message, text: &str, bot: &Bot, es_client: &EsHelper) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(":").map(|s| s.to_string()).collect();
    
    
    //let curr_mon = get_current_korean_time("%Y.%m.01");
    //println!("{:?}", curr_mon);
    

    Ok(())
}