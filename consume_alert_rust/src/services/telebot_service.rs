use crate::common::*;

use crate::models::consume_prodt_info::*;
use crate::models::consume_result_by_type::*;
use crate::models::document_with_id::*;
use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;

#[async_trait]
pub trait TelebotService {
    async fn tele_bot_send_msg(&self, msg: &str) -> Result<(), anyhow::Error>;
    async fn send_message_confirm(&self, msg: &str) -> Result<(), anyhow::Error>;
    async fn tele_bot_send_photo(&self, image_path: &str) -> Result<(), anyhow::Error>;
    async fn send_photo_confirm(&self, image_path_vec: &Vec<String>) -> Result<(), anyhow::Error>;

    async fn send_consumption_message<'life1, 'life2, 'msg, T>(
        &self,
        items: &'life1 Vec<T>,
        message_builder: fn(&'life2 T) -> String,
        empty_flag: bool,
        empty_msg: &'msg str,
        msg_title: &'msg str,
    ) -> Result<(), anyhow::Error>
    where
        'life1: 'life2,
        T: Send + Sync;

    async fn send_message_consume_split(
        &self,
        to_python_graph_line: &ToPythonGraphLine,
        consume_detail_list: &Vec<DocumentWithId<ConsumeProdtInfo>>,
    ) -> Result<(), anyhow::Error>;

    async fn send_message_consume_info_by_typelist(
        &self,
        type_consume_info: &Vec<ConsumeResultByType>,
        start_dt: NaiveDate,
        end_dt: NaiveDate,
        total_cost: f64,
    ) -> Result<(), anyhow::Error>;

    fn get_input_text(&self) -> String;

    async fn send_message_struct_info<T: Serialize + Sync>(
        &self,
        obj_struct: &T,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters)]
pub struct TelebotServicePub {
    pub bot: Arc<Bot>,
    pub chat_id: ChatId,
    pub input_text: String,
}

impl TelebotServicePub {
    #[doc = "Telegram 서비스 생성자"]
    /// # Arguments
    /// * `bot`     - Telegram Bot 객체
    /// * `message` - message 데이터 객체
    ///
    /// # Returns
    /// * Self
    pub fn new(bot: Arc<Bot>, message: Message) -> Self {
        let input_text = match message.text() {
            Some(input_text) => input_text,
            None => {
                error!("[Error][handle_commandhandle_command()] The entered value does not exist.");
                ""
            }
        }
        .to_string()
        .to_lowercase();

        let chat_id: ChatId = message.chat.id;

        Self {
            bot,
            chat_id,
            input_text,
        }
    }

    #[doc = "Generator for TEST"]
    /// # Arguments
    /// * `bot`         - Object Telegram bot
    /// * `input_str`   - String to be entered
    ///
    /// # Returns
    /// * Self
    pub fn new_test(bot: Arc<Bot>, input_str: &str) -> Self {
        let chat_id: ChatId = ChatId(5346196727);
        let input_text: String = input_str.to_string().to_lowercase();

        Self {
            bot,
            chat_id,
            input_text,
        }
    }

    #[doc = "Generic function to retry operations"]
    /// # Arguments
    /// * `operation` -
    /// * `max_retries` -
    /// * `retry_delay` -
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn try_send_operation<F, Fut>(
        &self,
        operation: F,
        max_retries: usize,
        retry_delay: Duration,
    ) -> Result<(), anyhow::Error>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<(), anyhow::Error>>,
    {
        let mut attempts = 0;

        while attempts <= max_retries {
            match operation().await {
                Ok(_) => return Ok(()),
                Err(e) if attempts == max_retries => {
                    error!(
                        "[Telebot Error][try_send_operation()] Max attempts reached. : {:?}",
                        e
                    );
                    return Err(e);
                }
                Err(e) => {
                    error!("{:?}", e);
                    thread::sleep(retry_delay);
                    attempts += 1;
                }
            }
        }

        Err(anyhow!(
            "[Telebot Error][try_send_operation()] Failed after retrying {} times",
            max_retries
        ))
    }
}

#[async_trait]
impl TelebotService for TelebotServicePub {
    #[doc = "This async function serializes a generic struct into a formatted string"]
    /// # Arguments
    /// * obj_struct - Distinguishing characters
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_message_struct_info<T: Serialize + Sync>(
        &self,
        obj_struct: &T,
    ) -> Result<(), anyhow::Error> {
        let obj_val: Value = serde_json::to_value(obj_struct).map_err(|err| {
            anyhow!(
                "[Error][convert_json_from_struct()] Failed to serialize struct to JSON: {}",
                err
            )
        })?;

        if let Some(obj) = obj_val.as_object() {
            let mut result_string = String::new();

            for (key, value) in obj {
                let mut value_str: String = String::from("");

                match value {
                    Value::Number(num) => {
                        if let Some(n) = num.as_i64() {
                            value_str = n.to_formatted_string(&Locale::ko);
                        }
                    }
                    _ => {
                        value_str = value.to_string();
                    }
                }

                result_string.push_str(&format!("{}: {}, \n", key, value_str));
            }

            if !result_string.is_empty() {
                for _n in 0..3 {
                    result_string.pop();
                }
            }

            self.send_message_confirm(&result_string).await?;
        } else {
            return Err(anyhow!("Parsed JSON is not an object"));
        }

        Ok(())
    }

    #[doc = "Send message via Telegram Bot"]
    async fn tele_bot_send_msg(&self, msg: &str) -> Result<(), anyhow::Error> {
        self.bot
            .send_message(self.chat_id, msg)
            .await
            .context("[Telebot Error][tele_bot_send_msg()] Failed to send command response.")?;

        Ok(())
    }

    #[doc = "Retry sending messages"]
    async fn send_message_confirm(&self, msg: &str) -> Result<(), anyhow::Error> {
        self.try_send_operation(|| self.tele_bot_send_msg(msg), 6, Duration::from_secs(40))
            .await
    }

    #[doc = "tele_bot_send_photo"]
    async fn tele_bot_send_photo(&self, image_path: &str) -> Result<(), anyhow::Error> {
        let photo: InputFile = InputFile::file(Path::new(image_path));
        self.bot
            .send_photo(self.chat_id, photo)
            .await
            .context("Telebot Error][tele_bot_send_photo()] Failed to send Photo.")?;

        Ok(())
    }

    #[doc = "Function that transfers pictures"]
    /// # Arguments
    /// * `image_path_vec` - Vector that elements the paths of a photo file
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_photo_confirm(&self, image_path_vec: &Vec<String>) -> Result<(), anyhow::Error> {
        for image_path in image_path_vec {
            self.try_send_operation(
                || self.tele_bot_send_photo(image_path),
                6,
                Duration::from_secs(40),
            )
            .await?;
        }

        Ok(())
    }

    #[doc = "Functions that send messages related to consumption details"]
    async fn send_consumption_message<'life1, 'life2, 'msg, T>(
        &self,
        items: &'life1 Vec<T>,
        message_builder: fn(&'life2 T) -> String,
        empty_flag: bool,
        empty_msg: &'msg str,
        msg_title: &'msg str,
    ) -> Result<(), anyhow::Error>
    where
        'life1: 'life2, /* 'life1' should live longer than 'life2' */
        T: Send + Sync,
    {
        let cnt: usize = 10;
        let items_len: usize = items.len();
        let loop_cnt: usize = (items_len / cnt) + (if items_len % cnt != 0 { 1 } else { 0 });

        if empty_flag {
            self.send_message_confirm(empty_msg).await?;
        } else {
            for idx in 0..loop_cnt {
                let mut send_text: String = String::new();
                let end_idx: usize = cmp::min(items_len, (idx + 1) * cnt);

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
    /// # Arguments
    /// * `to_python_graph_line` - Objects for drawing graphs
    /// * `consume_detail_list` - Consumption Details List
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_message_consume_split(
        &self,
        to_python_graph_line: &ToPythonGraphLine,
        consume_detail_list: &Vec<DocumentWithId<ConsumeProdtInfo>>,
    ) -> Result<(), anyhow::Error> {
        let start_dt: &String = to_python_graph_line.start_dt();
        let end_dt: &String = to_python_graph_line.end_dt();
        let total_cost: f64 = *to_python_graph_line.total_cost();
        let total_cost_i64: i64 = total_cost as i64;

        let empty_flag = if consume_detail_list.is_empty() {
            true
        } else {
            false
        };

        self.send_consumption_message(consume_detail_list, |item| {
            format!(
                "name : {}\ndate : {}\ncost : {}\n",
                item.source.prodt_name(),
                item.source.timestamp(),
                item.source.prodt_money().to_formatted_string(&Locale::ko)
            )},
            empty_flag,
            &format!("The money you spent from [{} ~ {}] is [ {} won ]\nThere is no consumption history to be viewed during that period.", start_dt, end_dt, total_cost_i64.to_formatted_string(&Locale::ko)),
            &format!("The money you spent from [{} ~ {}] is [ {} won ]\n=========[DETAIL]=========\n", start_dt, end_dt, total_cost_i64.to_formatted_string(&Locale::ko)) 
        ).await
    }

    #[doc = "Functions that return consumption aggregate information by category over a specific period of time"]
    /// # Arguments
    /// * `type_consume_info` - Consumption aggregation information by category
    /// * `start_dt`
    /// * `end_dt`
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_message_consume_info_by_typelist(
        &self,
        type_consume_info: &Vec<ConsumeResultByType>,
        start_dt: NaiveDate,
        end_dt: NaiveDate,
        total_cost: f64,
    ) -> Result<(), anyhow::Error> {
        let total_cost_i64: i64 = total_cost as i64;
        let empty_flag: bool = if type_consume_info.is_empty() {
            true
        } else {
            false
        };

        self.send_consumption_message(type_consume_info, |item| {
            format!(
                "category name : {}\ncost : {}\ncost(%) : {}%\n",
                item.consume_prodt_type(),
                item.consume_prodt_cost().to_formatted_string(&Locale::ko),
                item.consume_prodt_per()
            )},
            empty_flag,
            &format!("The money you spent from [{} ~ {}] is [ {} won ]\nThere is no consumption history to be viewed during that period.", start_dt, end_dt, total_cost_i64.to_formatted_string(&Locale::ko)),
            &format!("The money you spent from [{} ~ {}] is [ {} won ]\n=========[DETAIL]=========\n", start_dt, end_dt, total_cost_i64.to_formatted_string(&Locale::ko))
        ).await?;

        Ok(())
    }

    #[doc = "String entered through `Telegram`"]
    fn get_input_text(&self) -> String {
        self.input_text.to_string()
    }
}
