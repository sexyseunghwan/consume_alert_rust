use crate::common::*;

use crate::model::ConsumeInfo::*;
use crate::model::ConsumeTypeInfo::*;

#[async_trait]
pub trait TelebotService {
    
    async fn tele_bot_send_msg(&self, msg: &str) -> Result<(), anyhow::Error>;
    async fn send_message_confirm(&self, msg: &str) -> Result<(), anyhow::Error>;
    async fn tele_bot_send_photo(&self, image_path: &str) -> Result<(), anyhow::Error>;
    async fn send_photo_confirm(&self, image_path: &str) -> Result<(), anyhow::Error>;
    
    async fn send_consumption_message<'life1, 'life2, 'msg, T>
    (
        &self,
        items: &'life1 Vec<T>,  
        message_builder: fn(&'life2 T) -> String,
        empty_flag: bool,
        empty_msg: &'msg str,
        msg_title: &'msg str
    ) -> Result<(), anyhow::Error>
    where
    'life1: 'life2,
    T: Send + Sync;

    async fn send_message_consume_split
    (
        &self,
        consume_list: &Vec<ConsumeInfo>, 
        total_cost: f64, 
        start_dt: NaiveDate, 
        end_dt: NaiveDate,
        empty_flag: bool
    ) -> Result<(), anyhow::Error>;


    async fn send_message_consume_type(
        &self,
        consume_type_list: &Vec<ConsumeTypeInfo>, 
        total_cost: f64, 
        start_dt: NaiveDate, 
        end_dt: NaiveDate,
        empty_flag: bool
    ) -> Result<(), anyhow::Error>;


    async fn send_message_consume_type_list(
        &self,
        consume_type_list: &Vec<String>, 
        empty_flag: bool
    ) -> Result<(), anyhow::Error>;


    fn get_input_text(&self) -> String;

}


#[derive(Debug, Getters)]
pub struct TelebotServicePub
{
    pub bot: Bot,
    pub chat_id: ChatId,
    pub input_text: String 
}


impl TelebotServicePub {
        
    pub fn new(bot: Bot, message: Message) -> Self {

        let input_text = match message.text() {
            Some(input_text) => input_text,
            None => {
                error!("[Error][handle_commandhandle_command()] The entered value does not exist.");
                ""
            }
        }.to_string();

        let chat_id = message.chat.id;

        Self {
            bot, chat_id, input_text
        }
    }

    #[doc = "Generic function to retry operations"]
    async fn try_send_operation<F, Fut>(&self, operation: F, max_retries: usize, retry_delay: Duration) -> Result<(), anyhow::Error>
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
    

}

#[async_trait]
impl TelebotService for TelebotServicePub {

    #[doc = "Send message via Telegram Bot"]
    async fn tele_bot_send_msg(&self, msg: &str) -> Result<(), anyhow::Error> {
        
        self.bot.send_message(self.chat_id, msg)
            .await
            .context("[Telebot Error][tele_bot_send_msg()] Failed to send command response.")?;

        Ok(())
    }
    

    #[doc = "Retry sending messages"]
    async fn send_message_confirm(&self, msg: &str) -> Result<(), anyhow::Error> {
        self.try_send_operation(|| self.tele_bot_send_msg(msg), 6, Duration::from_secs(40)).await
    }


    #[doc = "tele_bot_send_photo"]
    async fn tele_bot_send_photo(&self, image_path: &str) -> Result<(), anyhow::Error> {
        
        let photo = InputFile::file(Path::new(image_path));
        self.bot.send_photo(self.chat_id, photo).await.context("Telebot Error][tele_bot_send_photo()] Failed to send Photo.")?;
        
        Ok(())
    }


    #[doc = "Retry sending photos"]
    async fn send_photo_confirm(&self, image_path: &str) -> Result<(), anyhow::Error> {
        self.try_send_operation(|| self.tele_bot_send_photo(image_path), 6, Duration::from_secs(40)).await
    }
    

    #[doc = "Functions that send messages related to consumption details"]
    async fn send_consumption_message<'life1, 'life2, 'msg, T>(
        &self, 
        items: &'life1 Vec<T>,  
        message_builder: fn(&'life2 T) -> String,
        empty_flag: bool,
        empty_msg: &'msg str,
        msg_title: &'msg str
    ) -> Result<(), anyhow::Error> 
    where
        'life1: 'life2, // 'life1' should live longer than 'life2'
        T: Send + Sync
    {
        
        //let total_cost_i32 = total_cost as i32;
        let cnt = 10;
        let items_len = items.len();
        let loop_cnt = (items_len / cnt) + (if items_len % cnt != 0 { 1 } else { 0 });
        
        if empty_flag {
            self.send_message_confirm(empty_msg).await?;
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
        
                self.send_message_confirm(&send_text).await?;
            }
        }    
    
    
        Ok(())
    }


    #[doc = "Functions that send messages related to consumption details"]
    async fn send_message_consume_split(
        &self,
        consume_list: &Vec<ConsumeInfo>, 
        total_cost: f64, 
        start_dt: NaiveDate, 
        end_dt: NaiveDate,
        empty_flag: bool
    ) -> Result<(), anyhow::Error> {

        let total_cost_i32 = total_cost as i32;

        self.send_consumption_message(consume_list, |item| {
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


    #[doc = "Function that sends messages related to consumption type history"]
    async fn send_message_consume_type(
        &self,
        consume_type_list: &Vec<ConsumeTypeInfo>, 
        total_cost: f64, 
        start_dt: NaiveDate, 
        end_dt: NaiveDate,
        empty_flag: bool
    ) -> Result<(), anyhow::Error> {

        let total_cost_i32 = total_cost as i32;

        self.send_consumption_message(consume_type_list, |item| {
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

    #[doc = "Function that sends "]    
    async fn send_message_consume_type_list(
        &self,
        consume_type_list: &Vec<String>, 
        empty_flag: bool
    ) -> Result<(), anyhow::Error> {

        self.send_consumption_message(consume_type_list, |item| {
            format!("{}\n",item.to_string())},
            empty_flag,
            "'consume_type' does not exist.",
            "ConsumeType List\n=========[DETAIL]=========\n"
        ).await

    } 

    #[doc = ""] 
    fn get_input_text(&self) -> String {
        self.input_text.to_string()
    }
}
