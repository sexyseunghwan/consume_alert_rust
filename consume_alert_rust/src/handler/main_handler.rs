use chrono::format::parse;

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
use crate::model::MealCheckIndex::*;
use crate::model::ConsumingIndexProdType::*;


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


    #[doc = "Function that preprocesses the text entered by telegram"]
    /// # Arguments
    /// * split_gubun - Distinguishing characters
    /// 
    /// # Returns
    /// * Vec<String> - Distinguishing String vector
    fn preprocess_string(&self, split_gubun: &str) -> Vec<String> {
        
        let args = self.telebot_service.get_input_text();
        let args_aplit = &args[2..];
        let split_args_vec: Vec<String> = args_aplit
            .split(split_gubun)
            .map(|s| s.trim().to_string())
            .collect();

        split_args_vec
    }


    #[doc = "Function reponsible for calculation consumption data and linking python api"]
    /// # Arguments
    /// * `permon_datetime` - Comparison group date data
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn process_calculate_and_post_python_api(&self, permon_datetime: PerDatetime) -> Result<(), anyhow::Error> {

        /* Consumption Type Information Vectors - Get all classification of consumption data `ex) Meals, cafes, etc...` */
        // let consume_type_vec: Vec<ProdtTypeInfo> = 
        //     self.db_service
        //         .get_classification_consumption_type("consuming_index_prod_type").await?;
        
        // let consume_type_map: HashMap<String, ConsumingIndexProdType> = 
        //     self.db_service
        //         .get_classification_consumption_type("consuming_index_prod_type").await?;
        
        let start = std::time::Instant::now(); // 시작 시간 측정

        let mut cur_consume_detail_infos = 
            self.db_service
                .get_consume_detail_specific_period( permon_datetime.date_start,  permon_datetime.date_end).await?;
        

        println!("{:?}", cur_consume_detail_infos.1);
        // let mut versus_consume_detail_infos = 
        //     self.db_service
        //         .get_consume_detail_specific_period( permon_datetime.n_date_start,  permon_datetime.n_date_end).await?;
        


        let duration = start.elapsed(); // 경과 시간 계산
        println!("Time elapsed in expensive_function() is: {:?}", duration);

        // let cur_mon_total_cost_infos = 
        //     self.db_service
        //         .total_cost_detail_specific_period(permon_datetime.date_start, permon_datetime.date_end, "consuming_index_prod_new", &consume_type_map).await?;
        
        // let pre_mon_total_cost_infos = 
        //     self.db_service
        //         .total_cost_detail_specific_period(permon_datetime.n_date_start, permon_datetime.n_date_end, "consuming_index_prod_new", &consume_type_map).await?;
        
        /* Python api */
        //self.command_common_double(cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;
        
        Ok(())
    } 
    

    #[doc = "command handler: Writes the expenditure details to the index in ElasticSearch. -> c"]
    async fn command_consumption(&self) -> Result<(), anyhow::Error> {

        let split_args_vec = self.preprocess_string(":");
        
        
        if split_args_vec.len() != 2 {

            self.telebot_service
                .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) c snack:15000")
                .await?;
            
            return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", self.telebot_service.get_input_text())));
        }
        
        let (consume_name, consume_cash) = (&split_args_vec[0], &split_args_vec[1]);        
        
        let consume_cash_i64: i64 = match get_parsed_value_from_vector(&split_args_vec, 1) {
            Ok(consume_cash_i64) => consume_cash_i64,
            Err(e) => {
                self.telebot_service
                    .send_message_confirm("The second parameter must be numeric. \nEX) c snack:15000")
                    .await?;

                return Err(anyhow!("[Parameter Error][command_consumption()] Non-numeric 'cash' parameter: {:?} : {:?}", consume_cash, e));
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

        let split_args_vec = self.preprocess_string(" ");

        println!("{:?}", split_args_vec);

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            
            1 => {
                let date_start = get_current_kor_naivedate_first_date()?;
                let date_end = get_lastday_naivedate(date_start)?;
                
                self.command_service.get_nmonth_to_current_date(date_start, date_end, -1)?
            },
            2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) => {

                let dates = split_args_vec[1].split('.').collect::<Vec<&str>>();

                let year: i32 = dates.get(0)
                    .ok_or_else(|| anyhow!("test"))?
                    .parse()?;
                
                let month: u32 = dates.get(1)
                    .ok_or_else(|| anyhow!("test"))?
                    .parse()?;
                
                //get_parsed_value_from_vector(&split_args_vec, 1)?;
                

                let date_start = get_naivedate(year, month, 1)?;
                let date_end = get_lastday_naivedate(date_start)?;

                self.command_service.get_nmonth_to_current_date(date_start, date_end, -1)?
            },
            _ => {
                self.telebot_service
                    .send_message_confirm("Invalid date format. Please use format YYYY.MM like cm 2023.07")
                    .await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_mon()] Invalid format of 'text' variable entered as parameter. : {:?}", self.telebot_service.get_input_text()));
            }
        };
        
        println!("{:?}", permon_datetime);

        self.process_calculate_and_post_python_api(permon_datetime).await?;

        Ok(())
    }



    #[doc = "command handler: Checks how much you have consumed during a specific periods -> ctr"]
    async fn command_consumption_per_term(&self) -> Result<(), anyhow::Error> {

        let split_args_vec = self.preprocess_string(" ");
        
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

                return Err(anyhow!("[Parameter Error][command_consumption_per_term()] Invalid format of 'text' variable entered as parameter. : {:?}", self.telebot_service.get_input_text()));           
            }
        };
        
        println!("{:?}", permon_datetime);

        self.process_calculate_and_post_python_api(permon_datetime).await?;

        Ok(())
    }
    


    #[doc = "command handler: Checks how much you have consumed during a day -> ct"]
    async fn command_consumption_per_day(&self) -> Result<(), anyhow::Error> {

        let split_args_vec = self.preprocess_string(" ");

        
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
                
                return Err(anyhow!("[Parameter Error][command_consumption_per_day()] Invalid format of 'text' variable entered as parameter. : {:?}", self.telebot_service.get_input_text()));
            }
        };

        println!("{:?}", permon_datetime);
        self.process_calculate_and_post_python_api(permon_datetime).await?;

        Ok(())
    }

    
    
    #[doc = "command handler: Check the consumption details from the date of payment to the next payment. -> cs"]    
    async fn command_consumption_per_salary(&self) -> Result<(), anyhow::Error> {

        let split_args_vec = self.preprocess_string(" ");

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
                    let curdate_str = format!("{}.01", &split_args_vec[1]);
                    let cur_date = NaiveDate::parse_from_str(&curdate_str, "%Y.%m.%d")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_salary()] This does not fit the date format : {:?}", e))?;
                    
                    let cur_date_end = get_naivedate(cur_date.year(), cur_date.month(), 25)?;
                    let cur_date_start = get_add_month_from_naivedate(cur_date_end, -1)?;
                    
                    self.command_service
                        .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
                },
            _ => {

                self.telebot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) cs or cs 2023.11").await?;
                
                return Err(anyhow!("[Parameter Error][command_consumption_per_day()] Invalid format of 'text' variable entered as parameter. : {:?}", self.telebot_service.get_input_text()));
            }
        };

        println!("{:?}", permon_datetime);

        self.process_calculate_and_post_python_api(permon_datetime).await?;
        
        Ok(())
    }

    

    #[doc = "command handler: Checks how much you have consumed during a week -> cw"]
    async fn command_consumption_per_week(&self) -> Result<(), anyhow::Error> {

        let split_args_vec = self.preprocess_string(" ");


        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {

                let now = get_current_kor_naive_datetime();
                let today = now.date();
                let weekday = today.weekday();

                let days_until_monday = Weekday::Mon.num_days_from_monday() as i64 - weekday.num_days_from_monday() as i64;
                let monday = today + chrono::Duration::days(days_until_monday);

                let date_start = monday + chrono::Duration::days(0);
                let date_end = monday + chrono::Duration::days(6);

                self.command_service
                    .get_nday_to_current_date(date_start, date_end, -7)?
            },
            _ => {
                self.telebot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) cw").await?;
                
                return Err(anyhow!("[Parameter Error][command_consumption_per_week()] Invalid format of 'text' variable entered as parameter. : {:?}", self.telebot_service.get_input_text()));
            }
        };

        println!("{:?}", permon_datetime);

        self.process_calculate_and_post_python_api(permon_datetime).await?;
        
        Ok(())
    }

    

    #[doc = "command handler: Checks how much you have consumed during one year -> cy"]
    async fn command_consumption_per_year(&self) -> Result<(), anyhow::Error> {
        
        let split_args_vec = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {

                let cur_year = get_current_kor_naivedate().year();
                let start_date = get_naivedate(cur_year, 1, 1)?;  
                let end_date = get_naivedate(cur_year, 12, 31)?; 
                
                self.command_service
                    .get_nmonth_to_current_date(start_date, end_date, -12)?

            },
            2 if split_args_vec
                .get(1)
                .map_or(false, |d| validate_date_format(d, r"^\d{4}$").unwrap_or(false)) => 
                {
                    let year = split_args_vec[1].parse::<i32>()?;
                    let start_date = get_naivedate(year, 1, 1)?; 
                    let end_date = get_naivedate(year, 12, 31)?;  

                    self.command_service
                        .get_nmonth_to_current_date(start_date, end_date, -12)?
                },
            _ => {
                
                self.telebot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX01) cy\nEX02) cy 2023").await?;
                
                return Err(anyhow!("[Parameter Error][command_consumption_per_year()] Invalid format of 'text' variable entered as parameter. : {:?}", self.telebot_service.get_input_text()));         
            }
        };

        println!("{:?}", permon_datetime);

        self.process_calculate_and_post_python_api(permon_datetime).await?;

        Ok(())
    }


    
    #[doc = "command handler: Function for recording meal times -> mc"]
    async fn command_record_fasting_time(&self) -> Result<(), anyhow::Error> {

        let split_args_vec = self.preprocess_string(" ");

        let meal_time = match split_args_vec.len() {
            1 => {
                get_current_kor_naive_datetime() 
            },
            2 if split_args_vec
                .get(1)
                .map_or(false, |d| validate_date_format(d, r"^\d{2}\:\d{2}$")
                .unwrap_or(false)) => {
                    
                    let naive_time = NaiveTime::parse_from_str(&format!("{}:00", split_args_vec[1]), "%H:%M:%S")
                        .map_err(|e| anyhow!("[Error][command_record_fasting_time()] problem occurred while converting the variable 'naive_time': {:?}", e))?;
                    
                    let cur_time = get_current_kor_naive_datetime();
                    let naive_datetime = 
                        get_naivedatetime(cur_time.year(), cur_time.month(), cur_time.day(), naive_time.hour(), naive_time.minute(), naive_time.second())
                            .map_err(|e| anyhow!("[Error][command_record_fasting_time()] problem occurred while converting the variable 'naive_datetime': {:?}", e))?;
                    
                    naive_datetime
                },
            _ => {
                self.telebot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX01) mc 22:30 \nEX02) mc").await?;
                
                return Err(anyhow!("[Parameter Error][command_record_fasting_time()] Invalid format of 'text' variable entered as parameter. : {:?}", self.telebot_service.get_input_text()));
            }
        };
        
        println!("meal_time: {:?}", meal_time);
        
        /* Brings the data of the most recent meal time of today's meal time. */
        let recent_mealtime_vec = self.db_service.get_recent_mealtime_data_from_elastic(1).await?; 
        let mealtime_data: MealCheckIndex;

        if recent_mealtime_vec.len() == 1 {
            let recent_mealtime_data = &recent_mealtime_vec[0];
            mealtime_data = MealCheckIndex::new(meal_time.to_string(), 0, recent_mealtime_data.laststamp() + 1);
        } else {   
            mealtime_data = MealCheckIndex::new(meal_time.to_string(), 0, 1);
        }
        
        self.db_service.post_model_to_es(MEAL_CHECK, mealtime_data).await?;
        
        Ok(())
    }


    
    async fn command_check_fasting_time(&self) -> Result<(), anyhow::Error> {

        Ok(())
    }

    async fn command_delete_fasting_time(&self) -> Result<(), anyhow::Error> {

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
        // self.telebot_service.send_message_consume_split(
        //     cur_consume_list, 
        //     cur_total_cost, 
        //     cur_start_dt, 
        //     cur_end_dt,
        //     cur_empty_flag
        // ).await?; 
        
        // if cur_total_cost > 0.0 { 

        //     // ( consumption type information, consumption type graph storage path )
        //     let comsume_type_infos = 
        //         self.graph_api_service.get_consume_type_graph(cur_total_cost, cur_start_dt, cur_end_dt, cur_consume_list).await?;
            
        //     let consume_type_list = &comsume_type_infos.0;
        //     let consume_type_img = comsume_type_infos.1;

        //     self.telebot_service.send_photo_confirm( &consume_type_img).await?;

        //     self.telebot_service.send_message_consume_type(
        //         consume_type_list, 
        //         cur_total_cost, 
        //         cur_start_dt, 
        //         cur_end_dt,
        //         cur_empty_flag).await?; 

        //     let delete_target_vec: Vec<String> = vec![consume_type_img];
        //     //delete_file(delete_target_vec)?;
        // }

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