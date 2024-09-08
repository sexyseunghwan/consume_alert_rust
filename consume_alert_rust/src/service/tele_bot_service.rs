use crate::{common::*, dtos::dto::*};


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
                error!("[Error] Max attempts reached. - try_send_operation() // {:?}", e);
                return Err(e)
            }
            Err(e) => {
                error!("{:?}", e);
                thread::sleep(retry_delay);
                attempts += 1;
            }
        }
    }
       
    Err(anyhow!("[Error] Failed after retrying {} times - try_send_operation()", max_retries))
}

/*
    Send message via Telegram Bot
*/
async fn tele_bot_send_msg(bot: &Bot, chat_id: ChatId, err_yn: bool, msg: &str) -> Result<(), anyhow::Error> {
    
    if ! err_yn { info!("{:?}", msg) }
    bot.send_message(chat_id, msg).await.context("[Error] Failed to send command response - tele_bot_send_msg()")?;
    
    Ok(())
}

/* 
    Retry sending messages
*/ 
pub async fn send_message_confirm(bot: &Bot, chat_id: ChatId, err_yn: bool, msg: &str) -> Result<(), anyhow::Error> {
    try_send_operation(|| tele_bot_send_msg(bot, chat_id, err_yn, msg), 6, Duration::from_secs(40)).await
}

/* 
    Send photo message via Telegram Bot
*/
async fn tele_bot_send_photo(bot: &Bot, chat_id: ChatId, image_path: &str) -> Result<(), anyhow::Error> {
    let photo = InputFile::file(Path::new(image_path));
    bot.send_photo(chat_id, photo).await.context("[Error] Failed to send Photo - tele_bot_send_photo()")?;
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
    total_cost: f64, 
    start_dt: NaiveDate, 
    end_dt: NaiveDate, 
    message_builder: fn(&T) -> String,
    empty_flag: bool
) -> Result<(), anyhow::Error> {
    
    let total_cost_i32 = total_cost as i32;
    let cnt = 10;
    let items_len = items.len();
    let loop_cnt = (items_len / cnt) + (if items_len % cnt != 0 { 1 } else { 0 });
    
    if empty_flag {
        send_message_confirm(
            bot, 
            chat_id, 
            false, 
            &format!("The money you spent from [{} ~ {}] is [ {} won ]\nThere is no consumption history to be viewed during that period.", start_dt, end_dt, total_cost_i32.to_formatted_string(&Locale::ko)), 
        ).await?;


    } else {
        
        for idx in 0..loop_cnt {
            let mut send_text = String::new();
            let end_idx = cmp::min(items_len, (idx + 1) * cnt);
    
            if idx == 0 {
                send_text.push_str(&format!("The money you spent from [{} ~ {}] is [ {} won ]\n=========[DETAIL]=========\n", start_dt, end_dt, total_cost_i32.to_formatted_string(&Locale::ko)));
            }
           
            for inner_idx in (cnt * idx)..end_idx {
                send_text.push_str("---------------------------------\n");
                send_text.push_str(&message_builder(&items[inner_idx]));
            }
    
            send_message_confirm(bot, chat_id, false, &send_text).await?;
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
    send_consumption_message(bot, chat_id, consume_list, total_cost, start_dt, end_dt, |item| {
        format!(
            "name : {}\ndate : {}\ncost : {}\n",
            item.prodt_name(),
            item.timestamp(),
            item.prodt_money().to_formatted_string(&Locale::ko)
        )},
        empty_flag
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
    send_consumption_message(bot, chat_id, consume_type_list, total_cost, start_dt, end_dt, |item| {
        format!(
            "category name : {}\ncost : {}\ncost(%) : {}%\n",
            item.prodt_type(),
            item.prodt_cost().to_formatted_string(&Locale::ko),
            item.prodt_per()
        )},
        empty_flag
    ).await
}




