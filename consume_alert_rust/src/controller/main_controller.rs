use crate::common::*;

use crate::services::elastic_query_service::*;
use crate::services::graph_api_service::*;
use crate::services::mysql_query_service::*;
use crate::services::telebot_service::*;

use crate::utils_modules::io_utils::*;
use crate::utils_modules::time_utils::*;

pub struct MainController<
    G: GraphApiService,
    E: ElasticQueryService,
    M: MysqlQueryService,
    T: TelebotService,
> {
    graph_api_service: Arc<G>,
    elastic_query_service: Arc<E>,
    mysql_query_service: Arc<M>,
    tele_bot_service: T,
}

impl<G: GraphApiService, E: ElasticQueryService, M: MysqlQueryService, T: TelebotService>
    MainController<G, E, M, T>
{
    pub fn new(
        graph_api_service: Arc<G>,
        elastic_query_service: Arc<E>,
        mysql_query_service: Arc<M>,
        tele_bot_service: T,
    ) -> Self {
        Self {
            graph_api_service,
            elastic_query_service,
            mysql_query_service,
            tele_bot_service,
        }
    }

    #[doc = "Function that processes the request when the request is received through telegram bot"]
    pub async fn main_call_function(&self) -> Result<(), anyhow::Error> {
        let input_text: String = self.tele_bot_service.get_input_text();

        match input_text.split_whitespace().next().unwrap_or("") {
            "c" => self.command_consumption().await?,
            // "cd" => self.command_delete_recent_cunsumption().await?,
            // "cm" => self.command_consumption_per_mon().await?,
            // "ctr" => self.command_consumption_per_term().await?,
            // "ct" => self.command_consumption_per_day().await?,
            // "cs" => self.command_consumption_per_salary().await?,
            // "cw" => self.command_consumption_per_week().await?,
            // "mc" => self.command_record_fasting_time().await?,
            // "mt" => self.command_check_fasting_time().await?,
            // "md" => self.command_delete_fasting_time().await?,
            // "cy" => self.command_consumption_per_year().await?,
            // "list" => self.command_get_consume_type_list().await?,
            _ => self.command_consumption_auto().await?, /* Basic Action */
        }

        Ok(())
    }

    #[doc = "Function that preprocesses the text entered by telegram"]
    /// # Arguments
    /// * split_gubun - Distinguishing characters
    ///
    /// # Returns
    /// * Vec<String> - Distinguishing String vector
    fn preprocess_string(&self, split_gubun: &str) -> Vec<String> {
        let args: String = self.tele_bot_service.get_input_text();
        let args_aplit: &str = &args[2..];
        let split_args_vec: Vec<String> = args_aplit
            .split(split_gubun)
            .map(|s| s.trim().to_string())
            .collect();

        split_args_vec
    }

    #[doc = "command handler: Writes the expenditure details to the index in ElasticSearch. -> c"]
    async fn command_consumption(&self) -> Result<(), anyhow::Error> {
        let split_args_vec: Vec<String> = self.preprocess_string(":");

        if split_args_vec.len() != 2 {
            self.tele_bot_service
                .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) c snack:15000")
                .await?;

            return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", self.tele_bot_service.get_input_text())));
        }

        let (consume_name, consume_cash) = (&split_args_vec[0], &split_args_vec[1]);

        let consume_cash_i64: i64 = match get_parsed_value_from_vector(&split_args_vec, 1) {
            Ok(consume_cash_i64) => consume_cash_i64,
            Err(e) => {
                self.tele_bot_service
                    .send_message_confirm(
                        "The second parameter must be numeric. \nEX) c snack:15000",
                    )
                    .await?;

                return Err(anyhow!("[Parameter Error][command_consumption()] Non-numeric 'cash' parameter: {:?} : {:?}", consume_cash, e));
            }
        };
        
        /* Set the product type here */
        let prodt_type: String = self
            .elastic_query_service
            .get_consume_type_judgement(consume_name)
            .await?;
        let cur_time: String = get_str_curdatetime();

        let con_index_prod: ConsumeIndexProdNew = ConsumeIndexProdNew::new(
            cur_time.clone(),
            cur_time.clone(),
            consume_name.to_string(),
            consume_cash_i64,
            prodt_type,
        );

        let document: Value = convert_json_from_struct(&con_index_prod)?;

        let es_client: EsRepositoryPub = get_elastic_conn()?;
        es_client.post_query(&document, CONSUME_DETAIL).await?;

        self.telebot_service
            .send_message_struct_info(&con_index_prod)
            .await?;


        Ok(())
    }

    #[doc = "command handler: Writes the expenditure details to the index in ElasticSearch."]
    pub async fn command_consumption_auto(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
