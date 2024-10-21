use crate::common::*;

use crate::service::command_service::*;
use crate::service::graph_api_service::*;

use crate::utils_modules::common_function::*;


pub struct MainHandler<G: GraphApiService, C: CommandService> {
    graph_api_service: G,
    command_service: C
}


impl<G: GraphApiService, C: CommandService> MainHandler<G, C> {

    pub fn new(graph_api_service: G, command_service: C) -> Self {
        Self {
            graph_api_service,
            command_service
        }
    }

    #[doc = "docs"]
    pub async fn main_call_function(&self) -> Result<(), anyhow::Error> {
        
        let input_text = self.command_service.get_input_text();

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

        let args = self.command_service.get_input_text();//.as_str()[2..];
        let args_aplit = &args[2..];

        let split_args_vec: Vec<String> = args_aplit.split(':').map(|s| s.to_string()).collect();
        let mut consume_name = "";
        let mut consume_cash = "";


        if split_args_vec.len() != 2 {

            

        } else {

            

        }


        // if split_args_vec.len() != 2 {
            
        //     send_message_confirm(&self.bot, 
        //                         self.message_id, 
        //                         "There is a problem with the parameter you entered. Please check again. \nEX) c snack:15000").await?;

        //     return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", self.input_text)));
        // } 
        
        // if let Some(cons_name) = split_args_vec.get(0) {

        //     if let Some(price) = split_args_vec.get(1) {

        //         if !is_numeric(price) {
        //             send_message_confirm(&self.bot, self.message_id, "The second parameter must be numeric. \nEX) c snack:15000").await?;
        //             return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", self.input_text)));
        //         }

        //         consume_name = cons_name;
        //         consume_cash = price;
        //     }        

        // } else {
            
        //     send_message_confirm(&self.bot, 
        //                     self.message_id, 
        //                     "There is a problem with the parameter you entered. Please check again. \nEX) c snack:15000").await?;

        //     return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", text)));
        // }
        
        // let curr_time = get_current_kor_naive_datetime();
        
        // let document = json!({
        //     "@timestamp": get_str_from_naive_datetime(curr_time),
        //     "prodt_name": consume_name,
        //     "prodt_money": convert_numeric(consume_cash)
        // });
        
        // let es_client = get_elastic_conn()?;
        // es_client.post_query(&document, "consuming_index_prod_new").await?;
        
        // Ok(())


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

    async fn command_consumption_auto(&self) -> Result<(), anyhow::Error> {

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