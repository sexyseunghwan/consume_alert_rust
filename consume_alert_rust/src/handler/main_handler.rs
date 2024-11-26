use crate::common::*;

use crate::repository::es_repository::*;

use crate::service::database_service::*;
use crate::service::graph_api_service::*;
use crate::service::tele_bot_service::*;
use crate::service::command_service::*;

use crate::utils_modules::common_function::*;
use crate::utils_modules::time_utils::*;
use crate::utils_modules::numeric_utils::*;

use crate::model::PerDatetime::*;
use crate::model::ProdtTypeInfo::*;
use crate::model::TotalCostInfo::*;


pub struct MainHandler<G: GraphApiService, D: DBService, T: TelebotService, C: CommandService> {
    graph_api_service: Arc<G>,
    db_service: Arc<D>,
    telebot_service: T,
    command_service: Arc<C>
}


impl<G: GraphApiService, D: DBService, T: TelebotService, C: CommandService> MainHandler<G, D, T, C> {

    pub fn new(graph_api_service: Arc<G>, db_service: Arc<D>, telebot_service: T, command_service: Arc<C>) -> Self {
        Self {
            graph_api_service,
            db_service,
            telebot_service,
            command_service
        }
    }
    
    #[doc = "Function that processes the request when the request is received through telegram bot"]
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

        let split_args: Vec<String> = args_aplit
            .split(':')
            .map(|s| s.trim().to_string())
            .collect();
        
        
        if split_args.len() != 2 {

            self.telebot_service
                .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) c snack:15000")
                .await?;
            
            return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", args)));
        }
        
        let (consume_name, consume_cash) = (&split_args[0], &split_args[1]);        
        
        let consume_cash_i64: i64 = match get_parsed_value_from_vector(&split_args, 1) {
            Ok(consume_cash_i64) => consume_cash_i64,
            Err(e) => {
                self.telebot_service
                    .send_message_confirm("The second parameter must be numeric. \nEX) c snack:15000")
                    .await?;

                return Err(anyhow!("[Parameter Error][command_consumption()] Non-numeric 'cash' parameter: {:?} :: {:?}", consume_cash, e));
            }
        };
        
        let document: Value = json!({
                "@timestamp": get_str_curdatetime(),
                "prodt_name": consume_name,
                "prodt_money": consume_cash_i64
            });
        
        let es_client = get_elastic_conn(); 
        es_client.post_query(&document, "consuming_index_prod_new_test").await?;
            
        Ok(())
    }
    
    

    #[doc = "command handler: Checks how much you have consumed during a month -> cm"]
    pub async fn command_consumption_per_mon(&self) -> Result<(), anyhow::Error> {

        let args = self.telebot_service.get_input_text();
        let args_aplit = &args[2..];
        let split_args_vec: Vec<String> = args_aplit.split(' ').map(String::from).collect();

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            
            1 => {
                let date_start = get_current_kor_naivedate_first_date()?;
                let date_end = get_lastday_naivedate(date_start)?;
                
                self.command_service.get_nmonth_to_current_date(date_start, date_end, -1)?
            },
            2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) => {
                
                let year: i32 = get_parsed_value_from_vector(&split_args_vec, 0)?;
                let month: u32 = get_parsed_value_from_vector(&split_args_vec, 1)?;
                
                let date_start = get_naivedate(year, month, 1)?;
                let date_end = get_lastday_naivedate(date_start)?;

                self.command_service.get_nmonth_to_current_date(date_start, date_end, -1)?
            },
            _ => {
                self.telebot_service
                    .send_message_confirm("Invalid date format. Please use format YYYY.MM like cm 2023.07")
                    .await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_mon()] Invalid format of 'text' variable entered as parameter. : {:?}", args));
            }
        };
        
        /* Consumption Type Information Vectors - Get all classification of consumption data `ex) Meals, cafes, etc...` */
        let consume_type_vec: Vec<ProdtTypeInfo> = 
            self.db_service
                .get_classification_consumption_type("consuming_index_prod_type").await?;
        
        let cur_mon_total_cost_infos = 
            self.db_service
                .total_cost_detail_specific_period(permon_datetime.date_start, permon_datetime.date_end, "consuming_index_prod_new", &consume_type_vec).await?;
        
        let pre_mon_total_cost_infos = 
            self.db_service
                .total_cost_detail_specific_period(permon_datetime.n_date_start, permon_datetime.n_date_end, "consuming_index_prod_new", &consume_type_vec).await?;
        
        
        println!("{:?}", cur_mon_total_cost_infos);
        println!("{:?}", pre_mon_total_cost_infos);
        
        /* Python api */
        //self.command_common_double(cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;
        
        Ok(())
    }



    #[doc = "command handler: Checks how much you have consumed during a specific periods -> ctr"]
    async fn command_consumption_per_term(&self) -> Result<(), anyhow::Error> {

        let args = self.telebot_service.get_input_text();
        let args_aplit = &args[2..];
        let split_args_vec: Vec<String> = args_aplit.split(' ').map(String::from).collect();
        
        let permon_datetime: PerDatetime = match split_args_vec.len() {

            2 if split_args_vec.get(1)
                .map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}$")
                .unwrap_or(false)) => 
                {
                    let dates = split_args_vec[1].split('-').collect::<Vec<&str>>();

                    let start_date = NaiveDate::parse_from_str(dates[0], "%Y.%m.%d")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_term()] This does not fit the date format : {:?}", e))?;
                    
                    let end_date = NaiveDate::parse_from_str(dates[1], "%Y.%m.%d")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_term()] This does not fit the date format : {:?}", e))?;

                    self.command_service
                        .get_nmonth_to_current_date(start_date, end_date, -1)?
                },
            _ => {
                self.telebot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) ctr 2023.07.07-2023.08.01")
                    .await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_term()] Invalid format of 'text' variable entered as parameter. : {:?}", args));           
            }
        };
        
        /* Consumption Type Information Vectors - Get all classification of consumption data `ex) Meals, cafes, etc...` */
        let consume_type_vec: Vec<ProdtTypeInfo> = 
            self.db_service
                .get_classification_consumption_type("consuming_index_prod_type").await?;
        
        let cur_mon_total_cost_infos = 
            self.db_service
                .total_cost_detail_specific_period(permon_datetime.date_start, permon_datetime.date_end, "consuming_index_prod_new", &consume_type_vec).await?;
        
        let pre_mon_total_cost_infos = 
            self.db_service
                .total_cost_detail_specific_period(permon_datetime.n_date_start, permon_datetime.n_date_end, "consuming_index_prod_new", &consume_type_vec).await?;
        

        /* Python api */
        //self.command_common_double(cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;

        Ok(())
    }
    


    #[doc = "command handler: Checks how much you have consumed during a day -> ct"]
    async fn command_consumption_per_day(&self) -> Result<(), anyhow::Error> {

        let args = self.telebot_service.get_input_text();
        let args_aplit = &args[2..];
        let split_args_vec: Vec<String> = args_aplit.split(' ').map(String::from).collect();

        
        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {
                
                let start_dt = get_current_kor_naivedate();
                let end_dt = get_current_kor_naivedate();
                
                self.command_service
                    .get_nday_to_current_date(start_dt, end_dt, -1)?
            },
            2 if split_args_vec
                .get(1)
                .map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}$")
                .unwrap_or(false)) => 
                {
                    let cur_date = NaiveDate::parse_from_str(&split_args_vec[1], "%Y.%m.%d")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_day()] This does not fit the date format : {:?}", e))?;
                    
                    self.command_service
                        .get_nday_to_current_date(cur_date, cur_date, -1)?
                },
            _ => {

                self.telebot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) ct or ct 2023.11.11").await?;
                
                return Err(anyhow!("[Parameter Error][command_consumption_per_day()] Invalid format of 'text' variable entered as parameter. : {:?}", args));
            }
        };

    
        /* Consumption Type Information Vectors - Get all classification of consumption data `ex) Meals, cafes, etc...` */
        let consume_type_vec: Vec<ProdtTypeInfo> = 
        self.db_service
            .get_classification_consumption_type("consuming_index_prod_type").await?;
    
        let cur_mon_total_cost_infos = 
            self.db_service
                .total_cost_detail_specific_period(permon_datetime.date_start, permon_datetime.date_end, "consuming_index_prod_new", &consume_type_vec).await?;
        
        let pre_mon_total_cost_infos = 
            self.db_service
                .total_cost_detail_specific_period(permon_datetime.n_date_start, permon_datetime.n_date_end, "consuming_index_prod_new", &consume_type_vec).await?;
        

        /* Python api */
        //self.command_common_double(cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;

        Ok(())
    }



    #[doc = "command handler: Check the consumption details from the date of payment to the next payment. -> cs"]    
    async fn command_consumption_per_salary(&self) -> Result<(), anyhow::Error> {

        let args = self.telebot_service.get_input_text();
        let args_aplit = &args[2..];
        let split_args_vec: Vec<String> = args_aplit.split(' ').map(String::from).collect();

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {
                
                let cur_day = get_current_kor_naivedate();
                let cur_year = cur_day.year();
                let cur_month = cur_day.month();
                let cur_date = cur_day.day();

                let cur_date_start  = if cur_date < 25 { 
                    let date = get_naivedate(cur_year, cur_month, 25)?;
                    get_add_month_from_naivedate(date, -1)?
                } else { 
                    get_naivedate(cur_year, cur_month, 25)?
                };
                
                let cur_date_end  = if cur_date < 25 { 
                    get_naivedate(cur_year, cur_month, 25)?
                } else { 
                    let date = get_naivedate(cur_year, cur_month, 25)?;
                    get_add_month_from_naivedate(date, 1)?
                };
                
                self.command_service
                    .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
                
            },
            2 if split_args_vec
                .get(1)
                .map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}$")
                .unwrap_or(false)) => 
                {
                    let cur_date = NaiveDate::parse_from_str(&split_args_vec[1], "%Y.%m")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_salary()] This does not fit the date format : {:?}", e))?;
                    
                    let cur_date_end = get_naivedate(cur_date.year(), cur_date.month(), 25)?;
                    let cur_date_start = get_add_month_from_naivedate(cur_date_end, -1)?;
                    
                    self.command_service
                        .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
                },
            _ => {

                self.telebot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) cs or cs 2023.11.11").await?;
                
                return Err(anyhow!("[Parameter Error][command_consumption_per_day()] Invalid format of 'text' variable entered as parameter. : {:?}", args));
            }
        };

        /* Consumption Type Information Vectors - Get all classification of consumption data `ex) Meals, cafes, etc...` */
        let consume_type_vec: Vec<ProdtTypeInfo> = 
        self.db_service
            .get_classification_consumption_type("consuming_index_prod_type").await?;
    
        let cur_mon_total_cost_infos = 
            self.db_service
                .total_cost_detail_specific_period(permon_datetime.date_start, permon_datetime.date_end, "consuming_index_prod_new", &consume_type_vec).await?;
        
        let pre_mon_total_cost_infos = 
            self.db_service
                .total_cost_detail_specific_period(permon_datetime.n_date_start, permon_datetime.n_date_end, "consuming_index_prod_new", &consume_type_vec).await?;
        

        /* Python api */
        //self.command_common_double(cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;
        
        Ok(())
    }


    
    #[doc = "command handler: Checks how much you have consumed during a week -> cw"]
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
    pub async fn command_consumption_auto(&self) -> Result<(), anyhow::Error> {

        let args = self.telebot_service.get_input_text();

        let re: Regex = Regex::new(r"\[.*?\]\n?")?; 
        let replace_string = re.replace_all(&args, "").to_string(); /* Remove the '[~]' string. */
        
        println!("replace_string:{}", replace_string);
        let split_args_vec: Vec<String> = replace_string
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(); /* It convert the string into an array */
        

        match self.command_service.process_by_consume_type(&split_args_vec).await {
            Ok(res) => res,
            Err(e) => {
                self.telebot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again.")
                    .await?;

                return Err(e)
            }
        }
        
        Ok(())
    }
    



    /* ==================================== Python API ==================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    #[doc = "Common Command Function Without Comparison"]
    async fn command_common_single(&self, cur_total_cost_infos: TotalCostInfo) -> Result<(), anyhow::Error> {

        let cur_total_cost = cur_total_cost_infos.total_cost;
        let cur_consume_list = cur_total_cost_infos.consume_list();
        let cur_empty_flag = cur_total_cost_infos.empty_flag;
        let cur_start_dt = cur_total_cost_infos.start_dt;
        let cur_end_dt = cur_total_cost_infos.end_dt;    

        // Hand over the consumption details to Telegram bot.
        self.telebot_service.send_message_consume_split(
            cur_consume_list, 
            cur_total_cost, 
            cur_start_dt, 
            cur_end_dt,
            cur_empty_flag
        ).await?; 
        
        if cur_total_cost > 0.0 { 

            // ( consumption type information, consumption type graph storage path )
            let comsume_type_infos = 
                self.graph_api_service.get_consume_type_graph(cur_total_cost, cur_start_dt, cur_end_dt, cur_consume_list).await?;
            
            let consume_type_list = &comsume_type_infos.0;
            let consume_type_img = comsume_type_infos.1;

            self.telebot_service.send_photo_confirm( &consume_type_img).await?;

            self.telebot_service.send_message_consume_type(
                consume_type_list, 
                cur_total_cost, 
                cur_start_dt, 
                cur_end_dt,
                cur_empty_flag).await?; 

            let delete_target_vec: Vec<String> = vec![consume_type_img];
            //delete_file(delete_target_vec)?;
        }

        Ok(())
    }
    

    #[doc = "docs"]
    async fn command_common_double(&self) -> Result<(), anyhow::Error> {


        Ok(())
    }
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */
    /* ==================================================================================== */

}