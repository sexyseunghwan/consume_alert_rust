use crate::common::*;

use crate::model::ConsumeInfo::*;
use crate::model::ConsumeTypeInfo::*;

pub struct tele_bot_service
{
    
}

/* 
    Generic function to retry operations
*/
async fn try_send_operation<F, Fut>(operation: F, max_retries: usize, retry_delay: Duration) -> Result<(), anyhow::Error>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<(), anyhow::Error>>,
{
    let mut attempts = 0;
    
    while attempts <= max_retries {

        match operation().await {
            Ok(_) => return Ok(()),
            Err(e) if attempts == max_retries => {
                error!("[Telebot Error][try_send_operation()] Max attempts reached. : {:?}", e);
                return Err(e)
            }
            Err(e) => {
                error!("{:?}", e);
                thread::sleep(retry_delay);
                attempts += 1;
            }
        }
    }
       
    Err(anyhow!("[Telebot Error][try_send_operation()] Failed after retrying {} times", max_retries))
}

/*
    Send message via Telegram Bot
*/
async fn tele_bot_send_msg(bot: &Bot, chat_id: ChatId, msg: &str) -> Result<(), anyhow::Error> {
    
    bot.send_message(chat_id, msg).await.context("[Telebot Error][tele_bot_send_msg()] Failed to send command response.")?;
    
    Ok(())
}

/* 
    Retry sending messages
*/ 
pub async fn send_message_confirm(bot: &Bot, chat_id: ChatId, msg: &str) -> Result<(), anyhow::Error> {
    try_send_operation(|| tele_bot_send_msg(bot, chat_id, msg), 6, Duration::from_secs(40)).await
}

/* 
    Send photo message via Telegram Bot
*/
async fn tele_bot_send_photo(bot: &Bot, chat_id: ChatId, image_path: &str) -> Result<(), anyhow::Error> {
    let photo = InputFile::file(Path::new(image_path));
    bot.send_photo(chat_id, photo).await.context("Telebot Error][tele_bot_send_photo()] Failed to send Photo.")?;
    Ok(())
}

/* 
    Retry sending photos
*/
pub async fn send_photo_confirm(bot: &Bot, chat_id: ChatId, image_path: &str) -> Result<(), anyhow::Error> {
    try_send_operation(|| tele_bot_send_photo(bot, chat_id, image_path), 6, Duration::from_secs(40)).await
}


/*
    Functions that send messages related to consumption details
*/
async fn send_consumption_message<T>(
    bot: &Bot, 
    chat_id: ChatId, 
    items: &Vec<T>,  
    message_builder: fn(&T) -> String,
    empty_flag: bool,
    empty_msg: &str,
    msg_title: &str
) -> Result<(), anyhow::Error> {
    
    //let total_cost_i32 = total_cost as i32;
    let cnt = 10;
    let items_len = items.len();
    let loop_cnt = (items_len / cnt) + (if items_len % cnt != 0 { 1 } else { 0 });
    
    if empty_flag {
        
        send_message_confirm(
            bot, 
            chat_id, 
            empty_msg, 
        ).await?;

    
    } else {
        
        for idx in 0..loop_cnt {
            let mut send_text = String::new();
            let end_idx = cmp::min(items_len, (idx + 1) * cnt);
    
            if idx == 0 {
                send_text.push_str(msg_title);
            }
           
            for inner_idx in (cnt * idx)..end_idx {
                send_text.push_str("---------------------------------\n");
                send_text.push_str(&message_builder(&items[inner_idx]));
            }
    
            send_message_confirm(bot, chat_id, &send_text).await?;
        }
    }    


    Ok(())
}



/*
    Functions that send messages related to consumption details  
*/
pub async fn send_message_consume_split(
    bot: &Bot, 
    chat_id: ChatId, 
    consume_list: &Vec<ConsumeInfo>, 
    total_cost: f64, 
    start_dt: NaiveDate, 
    end_dt: NaiveDate,
    empty_flag: bool
) -> Result<(), anyhow::Error> {

    let total_cost_i32 = total_cost as i32;

    send_consumption_message(bot, chat_id, consume_list, |item| {
        format!(
            "name : {}\ndate : {}\ncost : {}\n",
            item.prodt_name(),
            item.timestamp(),
            item.prodt_money().to_formatted_string(&Locale::ko)
        )},
        empty_flag,
        &format!("The money you spent from [{} ~ {}] is [ {} won ]\nThere is no consumption history to be viewed during that period.", start_dt, end_dt, total_cost_i32.to_formatted_string(&Locale::ko)),
        &format!("The money you spent from [{} ~ {}] is [ {} won ]\n=========[DETAIL]=========\n", start_dt, end_dt, total_cost_i32.to_formatted_string(&Locale::ko)) 
    ).await
}


/*
    Function that sends messages related to consumption type history
*/
pub async fn send_message_consume_type(
    bot: &Bot, 
    chat_id: ChatId, 
    consume_type_list: &Vec<ConsumeTypeInfo>, 
    total_cost: f64, 
    start_dt: NaiveDate, 
    end_dt: NaiveDate,
    empty_flag: bool
) -> Result<(), anyhow::Error> {

    let total_cost_i32 = total_cost as i32;

    send_consumption_message(bot, chat_id, consume_type_list, |item| {
        format!(
            "category name : {}\ncost : {}\ncost(%) : {}%\n",
            item.prodt_type(),
            item.prodt_cost().to_formatted_string(&Locale::ko),
            item.prodt_per()
        )},
        empty_flag,
        &format!("The money you spent from [{} ~ {}] is [ {} won ]\nThere is no consumption history to be viewed during that period.", start_dt, end_dt, total_cost_i32.to_formatted_string(&Locale::ko)),
        &format!("The money you spent from [{} ~ {}] is [ {} won ]\n=========[DETAIL]=========\n", start_dt, end_dt, total_cost_i32.to_formatted_string(&Locale::ko))
    ).await
}


/*
    
*/
pub async fn send_message_consume_type_list(
    bot: &Bot, 
    chat_id: ChatId, 
    consume_type_list: &Vec<String>, 
    empty_flag: bool
) -> Result<(), anyhow::Error> {
    
    send_consumption_message(bot, chat_id, consume_type_list, |item| {
        format!("{}\n",item.to_string())},
        empty_flag,
        "'consume_type' does not exist.",
        "ConsumeType List\n=========[DETAIL]=========\n"
    ).await
}