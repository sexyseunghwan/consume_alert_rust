use crate::common::*;


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
                error!("Max attempts reached. Last error: {:?}", e);
                return Err(e)
            }
            Err(e) => {
                error!("{:?}", e);
                thread::sleep(retry_delay);
                attempts += 1;
            }
        }
    }
    Err(anyhow::anyhow!("Failed after retrying {} times", max_retries))
}

// Send message via Telegram Bot
async fn tele_bot_send_msg(bot: &Bot, chat_id: ChatId, err_yn: bool, msg: &str, log_msg: &str) -> Result<(), anyhow::Error> {
    bot.send_message(chat_id, msg).await.context("Failed to send command response")?;
    if !err_yn {
        info!("{:?}", log_msg);
    }
    Ok(())
}

// Retry sending messages
pub async fn send_message_confirm(bot: &Bot, chat_id: ChatId, err_yn: bool, msg: &str, log_msg: &str) -> Result<(), anyhow::Error> {
    try_send_operation(|| tele_bot_send_msg(bot, chat_id, err_yn, msg, log_msg), 6, Duration::from_secs(40)).await
}

// Send photo message via Telegram Bot
async fn tele_bot_send_photo(bot: &Bot, chat_id: ChatId, image_path: &str) -> Result<(), anyhow::Error> {
    let photo = InputFile::file(Path::new(image_path));
    bot.send_photo(chat_id, photo).await.context("Failed to send Photo")?;
    Ok(())
}

// Retry sending photos
pub async fn send_photo_confirm(bot: &Bot, chat_id: ChatId, image_path: &str) -> Result<(), anyhow::Error> {
    try_send_operation(|| tele_bot_send_photo(bot, chat_id, image_path), 6, Duration::from_secs(40)).await
}






// /*
//     Function to send result message via Telegram Bot
// */
// async fn tele_bot_send_msg(bot: &Bot, chat_id: ChatId, err_yn: bool, msg: &str, log_msg: &str) -> Result<(), anyhow::Error> {
    
//     if err_yn {

//         bot.send_message(chat_id, msg)
//             .await
//             .context("Failed to send command response")?;
        
//     } else {
        
//         bot.send_message(chat_id, msg)
//             .await
//             .context("Failed to send command response")?;
        
//         info!("{:?}", log_msg);
//     }
    
//     Ok(())
// }


// /*
//     Functions that attempt to relay messages from telegram bot until successful (up to 6 times)
// */
// pub async fn send_message_confirm(bot: &Bot, chat_id: ChatId, err_yn: bool, msg: &str, log_msg: &str) -> Result<(), anyhow::Error> {

//     let mut flag = true;
//     let mut try_cnt = 0;
    
//     while flag {
        
//         if try_cnt > 6 { 
//             error!("You can no longer connect to telegram bot.");
//             break; 
//         }
        
//         match tele_bot_send_msg(bot, chat_id, err_yn, msg, log_msg).await {
//             Ok(_) => {
//                 flag = false;
//                 ()
//             },
//             Err(e) => {
//                 error!("{:?}", e);
//                 try_cnt += 1;
//                 thread::sleep(Duration::from_secs(40));
//             }
//         };
//     }
    
//     Ok(())

// }


// pub async fn send_photo_confirm(bot: &Bot, chat_id: ChatId, image_path: &str) -> Result<(), anyhow::Error> {

//     let mut flag = true;
//     let mut try_cnt = 0;
    
//     while flag {
        
//         if try_cnt > 6 { 
//             error!("You can no longer connect to telegram bot.");
//             break; 
//         }
        
//         match tele_bot_send_photo(bot, chat_id, image_path).await {
//             Ok(_) => {
//                 flag = false;
//                 ()
//             },
//             Err(e) => {
//                 error!("{:?}", e);
//                 try_cnt += 1;
//                 thread::sleep(Duration::from_secs(40));
//             }
//         };
//     }
    
//     Ok(())

// }

// /*
//     Function to send photo message via Telegram Bot
// */
// pub async fn tele_bot_send_photo(bot: &Bot, chat_id: ChatId, image_path: &str) -> Result<(), anyhow::Error> {

//     let photo = InputFile::file(Path::new(image_path));

//     bot.send_photo(chat_id, photo)
//         .await
//         .context("Failed to send Photo")?;
    
//     Ok(())
// }