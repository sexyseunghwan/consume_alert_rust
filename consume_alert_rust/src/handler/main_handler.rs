use crate::common::*;

use crate::repository::es_repository::*;

use crate::service::calculate_service::*;
use crate::service::graph_api_service::*;
use crate::service::tele_bot_service::*;
use crate::service::command_service::*;

use crate::utils_modules::common_function::*;
use crate::utils_modules::time_utils::*;
use crate::utils_modules::numeric_utils::*;


pub struct MainHandler<G: GraphApiService, C: CalculateService, T: TelebotService, CS: CommandService> {
    graph_api_service: Arc<G>,
    calculate_service: Arc<C>,
    telebot_service: T,
    command_service: Arc<CS>
}


impl<G: GraphApiService, C: CalculateService, T: TelebotService, CS: CommandService> MainHandler<G, C, T, CS> {

    pub fn new(graph_api_service: Arc<G>, calculate_service: Arc<C>, telebot_service: T, command_service: Arc<CS>) -> Self {
        Self {
            graph_api_service,
            calculate_service,
            telebot_service,
            command_service
        }
    }
    
    #[doc = "docs"]
    pub async fn main_call_function(&self) -> Result<(), anyhow::Error> {
        
        let input_text = self.telebot_service.get_input_text();

        if input_text.starts_with("c ") {
            self.command_consumption().await?;
        }
        else if input_text.starts_with("cm") {
            self.command_consumption_per_mon().await?;
        }
        else if input_text.starts_with("ctr") {
            self.command_consumption_per_term().await?;
        }
        else if input_text.starts_with("ct") {
            self.command_consumption_per_day().await?;
        }
        else if input_text.starts_with("cs") {
            self.command_consumption_per_salary().await?;
        }
        else if input_text.starts_with("cw") {
            self.command_consumption_per_week().await?;
        }
        else if input_text.starts_with("mc") {
            self.command_record_fasting_time().await?;
        }
        else if input_text.starts_with("mt") {
            self.command_check_fasting_time().await?;
        }
        else if input_text.starts_with("md") {
            self.command_delete_fasting_time().await?;
        }
        else if input_text.starts_with("cy") {
            self.command_consumption_per_year().await?;
        }
        else if input_text.starts_with("list") {
            self.command_get_consume_type_list().await?;
        }
        else 
        {
            self.command_consumption_auto().await?;
        }

        Ok(())
    }
    
    
    #[doc = "command handler: Writes the expenditure details to the index in ElasticSearch. -> c"]
    async fn command_consumption(&self) -> Result<(), anyhow::Error> {

        let args = self.telebot_service.get_input_text();
        let args_aplit = &args[2..];

        let split_args: Vec<String> = args_aplit.split(':').map(|s| s.to_string()).collect();

        if split_args.len() != 2 {

            self.telebot_service
                .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) c snack:15000")
                .await?;
            
            return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", args)));

        }

        let (consume_name, consume_cash) = (&split_args[0], &split_args[1]);        
        
        if !is_numeric(consume_cash.as_str()) {
            self.telebot_service
                .send_message_confirm("The second parameter must be numeric. \nEX) c snack:15000")
                .await?;
            
            return Err(anyhow!("[Parameter Error][command_consumption()] Non-numeric 'cash' parameter: {:?}", consume_cash));
        }

        let curr_time = get_current_kor_naive_datetime();

        let document = json!({
                "@timestamp": get_str_from_naive_datetime(curr_time),
                "prodt_name": consume_name,
                "prodt_money": convert_numeric(consume_cash.as_str())
            });
        
        let es_client = get_elastic_conn(); 
        es_client.post_query(&document, "consuming_index_prod_new").await?;

        Ok(())
    }


    
    async fn command_consumption_per_mon(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_consumption_per_term(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_consumption_per_day(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_consumption_per_salary(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_consumption_per_week(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_record_fasting_time(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_check_fasting_time(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_delete_fasting_time(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_consumption_per_year(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_get_consume_type_list(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }


    #[doc = "command handler: Writes the expenditure details to the index in ElasticSearch."]
    async fn command_consumption_auto(&self) -> Result<(), anyhow::Error> {

        let args = self.telebot_service.get_input_text();

        println!("{:?}", args);

        let re = Regex::new(r"\[.*?\]\n?")?;
        let repalce_string = re.replace_all(&args, "").to_string();

        let split_args_vec: Vec<String> = repalce_string.split('\n').map(|s| s.to_string()).collect();

        println!("{:?}", split_args_vec);

        let consume_type = split_args_vec
            .get(0)
            .ok_or_else(|| anyhow!("[Parameter Error][command_consumption_auto()] Invalid format of 'text' variable entered as parameter : {:?}", split_args_vec))?;
        
        println!("{:?}", consume_type);

        // if consume_type.contains("nh") {

        //     println!("{:?}", split_args_vec);    

            
            

        // } else if consume_type.contains("삼성") {

        //     println!("{:?}", split_args_vec);    
            

        // } else {

        //     self.telebot_service
        //         .send_message_confirm( "There is a problem with the parameter you entered. Please check again.")
        //         .await?;

        //     return Err(anyhow!(format!("[Parameter Error][command_consumption_auto()] Invalid format of 'text' variable entered as parameter. : {:?}", split_args_vec)));
        // }
        
        
        Ok(())
    }

}

/*
    Functions that handle each command
*/
// pub async fn handle_command(message: Message, bot: Bot) -> Result<(), anyhow::Error> {

//     let command_service = CommandService::new(bot, message)?;    
//     let input_text = command_service.input_text;
    
//     if input_text.starts_with("c ") {
//         command_service.command_consumption().await?;
//     }
//     else if input_text.starts_with("cm") {
//         command_service.command_consumption_per_mon().await?;
//     }
//     else if input_text.starts_with("ctr") {
//         command_service.command_consumption_per_term().await?;
//     }
//     else if input_text.starts_with("ct") {
//         command_service.command_consumption_per_day().await?;
//     }
//     else if input_text.starts_with("cs") {
//         command_service.command_consumption_per_salary().await?;
//     }
//     else if input_text.starts_with("cw") {
//         command_service.command_consumption_per_week().await?;
//     }
//     else if input_text.starts_with("mc") {
//         command_service.command_record_fasting_time().await?;
//     }
//     else if input_text.starts_with("mt") {
//         command_service.command_check_fasting_time().await?;
//     }
//     else if input_text.starts_with("md") {
//         command_service.command_delete_fasting_time().await?;
//     }
//     else if input_text.starts_with("cy") {
//         command_service.command_consumption_per_year().await?;
//     }
//     else if input_text.starts_with("list") {
//         command_service.command_get_consume_type_list().await?;
//     }
//     else 
//     {
//         command_service.command_consumption_auto().await?;
//     }
    
//     Ok(())
// }