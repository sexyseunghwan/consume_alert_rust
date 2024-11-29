use crate::common::*;

use crate::service::database_service::*;
use crate::service::tele_bot_service::*;

use crate::utils_modules::numeric_utils::*;
use crate::utils_modules::time_utils::*;
use crate::utils_modules::file_manager_utils::*;
use crate::utils_modules::common_function::*;

use crate::model::TotalCostInfo::*;
use crate::model::ToPythonGraphLine::*;
use crate::model::ProdtTypeInfo::*;
use crate::model::PerDatetime::*;
use crate::model::ConsumeIndexProd::*;
use crate::model::ConsumingIndexProdType::*;

use crate::repository::es_repository::*;


#[async_trait]
pub trait CommandService {
    
    /* inner Common Service */
    fn get_consume_time(&self, consume_time_name_vec: &Vec<String>) -> Result<String, anyhow::Error>;
    fn get_string_vector_by_replace(&self, intput_str: &str, replacements: &Vec<&str>) -> Result<Vec<String>, anyhow::Error>;
    fn get_consume_prodt_money(&self, consume_price_vec: &Vec<String>, idx: usize) -> Result<i32, anyhow::Error>;
    
    /* Service */
    async fn process_by_consume_type(&self, split_args_vec: &Vec<String>) -> Result<(), anyhow::Error>;
    fn get_consume_prodt_name(&self, consume_time_name_vec: &Vec<String>, idx: usize) -> Result<String, anyhow::Error>;
    fn get_nmonth_to_current_date(&self, date_start: NaiveDate, date_end: NaiveDate, nmonth: i32) -> Result<PerDatetime, anyhow::Error>;
    fn get_nday_to_current_date(&self, date_start: NaiveDate, date_end: NaiveDate, nday: i32) -> Result<PerDatetime, anyhow::Error>;
    //async fn calculate_total_cost_info<'a>(&self, consume_map: &'a HashMap<String, ConsumingIndexProdType>, consume_index_prod_vector: &'a mut Vec<ConsumeIndexProd>) -> Result<TotalCostInfo, anyhow::Error>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct CommandServicePub;


#[async_trait]
impl CommandService for CommandServicePub {
    
    
    #[doc = "Process processing function based on the type of payment"]
    /// # Arguments
    /// * `split_args_vec` - Array with strings as elements : Payment-related information vector: 
    /// - ex) ["nh카드3*3*승인", "신*환", "5,500원 일시불", "11/25 10:02", "메가엠지씨커피 선릉", "총누적469,743원"]
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn process_by_consume_type(&self, split_args_vec: &Vec<String>) -> Result<(), anyhow::Error> {

        let consume_type = split_args_vec
            .get(0)
            .ok_or_else(|| anyhow!("[Parameter Error][process_by_consume_type()] Invalid format of 'text' variable entered as parameter : {:?}", split_args_vec))?;
        
        let es_client = get_elastic_conn();
        let split_val = vec![",", "원"];
        let document: Value;
        
        if consume_type.contains("nh") {
            
            let consume_price_vec: Vec<String> = self.get_string_vector_by_replace(split_args_vec
                .get(2)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed. : {:?}", 2, split_args_vec))?,
                &split_val
            )?;
            
            let consume_price = self.get_consume_prodt_money(&consume_price_vec, 0)?;

            let consume_time_vec: Vec<String> = split_args_vec
                .get(3)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 3))?
                .split(" ")
                .map(|s| s.trim().to_string())
                .collect();
            
            let consume_time = self.get_consume_time(&consume_time_vec)?;

            let consume_name = split_args_vec
                .get(4)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'split_args_vec' vector was accessed.", 4))?;
            
            document = json!({
                "@timestamp": consume_time,
                "prodt_name": consume_name,
                "prodt_money": consume_price
            });
            
        } else if consume_type.contains("삼성"){

            let consume_price_vec = self.get_string_vector_by_replace(split_args_vec
                .get(1)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed. : {:?}", 1, split_args_vec))?,
                &split_val
            )?;
            
            let consume_price = self.get_consume_prodt_money(&consume_price_vec, 0)?;

            let consume_time_vec: Vec<String> = split_args_vec
                .get(2)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 2))?
                .split(" ")
                .map(|s| s.to_string())
                .collect();
            
            let consume_time = self.get_consume_time(&consume_time_vec)?;

            let consume_name = consume_time_vec
                .get(2)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 2))?;
            
            document = json!({
                "@timestamp": consume_time,
                "prodt_name": consume_name,
                "prodt_money": consume_price
            });
            
        }  
        else {
            return Err(anyhow!("[Error][process_by_consume_type()] Variable 'consume_type' contains an undefined string."))
        }
        
        es_client.post_query(&document, "consuming_index_prod_new_test").await?;
        
        Ok(())
    }



    #[doc = "Functions that vectorize by spaces, excluding certain characters from a string"]
    /// # Arguments
    /// * `intput_str`  - Applied String : ex) "289,545원 일시불"
    /// * `replacements`- Character vector to replace : ex) [",", "원"]
    /// 
    /// # Returns
    /// * Result<Vec<String>, anyhow::Error> - ex) ["289545", "일시불"]
    fn get_string_vector_by_replace(&self, intput_str: &str, replacements: &Vec<&str>) -> Result<Vec<String>, anyhow::Error> {
        
        let consume_price_vec: Vec<String> = intput_str
            .to_string()
            .split_whitespace()
            .map(|s| {
                replacements
                    .iter()
                    .fold(s.to_string(), |acc, replace| acc.replace(replace, ""))
            })
            .collect();

        Ok(consume_price_vec)
    }

    
    #[doc = "Function that parses date data from consumption data"]
    /// # Arguments
    /// * `consume_time_name_vec` - Vector with date, time data : ex) ["11/25", "10:02"]
    /// 
    /// # Returns
    /// * Result<String, anyhow::Error>
    fn get_consume_time(&self, consume_time_name_vec: &Vec<String>) -> Result<String, anyhow::Error> {

        /* "11/25" */
        let parsed_date = consume_time_name_vec
            .get(0)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_time()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", 0))?;

        /* "10:02" */
        let parsed_time = consume_time_name_vec
            .get(1)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_time()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", 1))?;
        
        let cur_year = get_current_kor_naivedate().year();
        let formatted_date_str = format!("{}/{}", cur_year, parsed_date);
        let format_date = NaiveDate::parse_from_str(&formatted_date_str, "%Y/%m/%d")?;
        let format_time = NaiveTime::parse_from_str(&parsed_time, "%H:%M")?;
        let format_datetime = NaiveDateTime::new(format_date, format_time);

        Ok(format_datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    }
    

    #[doc = "Function that parses the name of the consumed product"]
    fn get_consume_prodt_name(&self, consume_time_name_vec: &Vec<String>, idx: usize) -> Result<String, anyhow::Error> {

        let consume_name = consume_time_name_vec
            .get(idx)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_prodt_name()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", idx))?
            .trim();
        
        Ok(consume_name.to_string())
    }
    

    #[doc = "Function that parses the money spent"]
    fn get_consume_prodt_money(&self, consume_price_vec: &Vec<String>, idx: usize) -> Result<i32, anyhow::Error> {

        let consume_price = consume_price_vec
            .get(idx)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_prodt_money()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed.", idx))?
            .parse::<i32>()?;
        
        Ok(consume_price)
    }

    
    
    #[doc = "Function that returns the time allotted as a parameter and the time before/after `N` months"]
    /// # Arguments
    /// * `date_start`  
    /// * `date_end`    
    /// * `nmonth` - Before or after `N` months
    /// 
    /// # Returns
    /// * Result<PermonDatetime, anyhow::Error>
    fn get_nmonth_to_current_date(&self, date_start: NaiveDate, date_end: NaiveDate, nmonth: i32) -> Result<PerDatetime, anyhow::Error> {

        let n_month_start = get_add_month_from_naivedate(date_start, nmonth)?;
        let n_month_end = get_add_month_from_naivedate(date_end, nmonth)?;

        let per_mon_datetim = PerDatetime::new(date_start, date_end, n_month_start, n_month_end);
        
        Ok(per_mon_datetim)
    }
    


    #[doc = "Function that returns the time allotted as a parameter and the time before/after `N` days"]
    /// # Arguments
    /// * `date_start`  
    /// * `date_end`    
    /// * `nday` - Before or after `N` days
    /// 
    /// # Returns
    /// * Result<PermonDatetime, anyhow::Error>
    fn get_nday_to_current_date(&self, date_start: NaiveDate, date_end: NaiveDate, nday: i32) -> Result<PerDatetime, anyhow::Error> {

        let n_day_start = get_add_date_from_naivedate(date_start, nday)?;
        let n_day_end = get_add_date_from_naivedate(date_end, nday)?;

        let per_day_datetim = PerDatetime::new(date_start, date_end, n_day_start, n_day_end);
        
        Ok(per_day_datetim)
    }
    

    // #[doc = ""]
    // async fn calculate_total_cost_info<'a>(&self, consume_map: &'a HashMap<String, ConsumingIndexProdType>, consume_index_prod_vector: &'a mut Vec<ConsumeIndexProd>) -> Result<TotalCostInfo, anyhow::Error> {

    //     for elem in consume_index_prod_vector {

    //         let prodt_type
    //         let prodt_name = elem.prodt_name();

    //         //let test = consume_map.get(prodt_name);
    //         if let Some(test) = consume_map.get(prodt_name) {

    //         }

    //     }

    // }


    // #[doc = "Common Command Function Without Comparison"]
    // async fn command_common_single(&self, cur_total_cost_infos: TotalCostInfo) -> Result<(), anyhow::Error> {
        
    //     let cur_total_cost = cur_total_cost_infos.total_cost;
    //     let cur_consume_list = cur_total_cost_infos.consume_list();
    //     let cur_empty_flag = cur_total_cost_infos.empty_flag;
    //     let cur_start_dt = cur_total_cost_infos.start_dt;
    //     let cur_end_dt = cur_total_cost_infos.end_dt;    

    //     // Hand over the consumption details to Telegram bot.
    //     send_message_consume_split(
    //         &self.bot, 
    //         self.message_id, 
    //         cur_consume_list, 
    //         cur_total_cost, 
    //         cur_start_dt, 
    //         cur_end_dt,
    //         cur_empty_flag
    //     ).await?; 
        
    //     if cur_total_cost > 0.0 { 

    //         // ( consumption type information, consumption type graph storage path )
    //         let comsume_type_infos = get_consume_type_graph(cur_total_cost, cur_start_dt, cur_end_dt, cur_consume_list).await?;
    //         let consume_type_list = &comsume_type_infos.0;
    //         let consume_type_img = comsume_type_infos.1;

    //         send_photo_confirm(&self.bot, self.message_id, &consume_type_img).await?;

    //         send_message_consume_type(&self.bot, 
    //             self.message_id, 
    //             consume_type_list, 
    //             cur_total_cost, 
    //             cur_start_dt, 
    //             cur_end_dt,
    //             cur_empty_flag).await?; 

    //         let delete_target_vec: Vec<String> = vec![consume_type_img];
    //         delete_file(delete_target_vec)?;
    //     }

    //     Ok(())
    // }


    
}

// impl CommandService {
    

    
    // /*
    //     Common Command Function Without Comparison
    // */
    // async fn command_common_single(&self, cur_total_cost_infos: TotalCostInfo) -> Result<(), anyhow::Error> {
        
    //     let cur_total_cost = cur_total_cost_infos.total_cost;
    //     let cur_consume_list = cur_total_cost_infos.consume_list();
    //     let cur_empty_flag = cur_total_cost_infos.empty_flag;
    //     let cur_start_dt = cur_total_cost_infos.start_dt;
    //     let cur_end_dt = cur_total_cost_infos.end_dt;    

    //     // Hand over the consumption details to Telegram bot.
    //     send_message_consume_split(
    //         &self.bot, 
    //         self.message_id, 
    //         cur_consume_list, 
    //         cur_total_cost, 
    //         cur_start_dt, 
    //         cur_end_dt,
    //         cur_empty_flag
    //     ).await?; 
        
    //     if cur_total_cost > 0.0 { 

    //         // ( consumption type information, consumption type graph storage path )
    //         let comsume_type_infos = get_consume_type_graph(cur_total_cost, cur_start_dt, cur_end_dt, cur_consume_list).await?;
    //         let consume_type_list = &comsume_type_infos.0;
    //         let consume_type_img = comsume_type_infos.1;

    //         send_photo_confirm(&self.bot, self.message_id, &consume_type_img).await?;

    //         send_message_consume_type(&self.bot, 
    //             self.message_id, 
    //             consume_type_list, 
    //             cur_total_cost, 
    //             cur_start_dt, 
    //             cur_end_dt,
    //             cur_empty_flag).await?; 

    //         let delete_target_vec: Vec<String> = vec![consume_type_img];
    //         delete_file(delete_target_vec)?;
    //     }

    //     Ok(())
    // }


//     /*
//         Common command function with comparison group
//     */
//     async fn command_common_double(&self, cur_total_cost_infos: TotalCostInfo, pre_total_cost_infos: TotalCostInfo) -> Result<(), anyhow::Error> {
        

//         let cur_total_cost = cur_total_cost_infos.total_cost;
//         let cur_consume_list = cur_total_cost_infos.consume_list();
//         let cur_empty_flag = cur_total_cost_infos.empty_flag;
//         let cur_start_dt = cur_total_cost_infos.start_dt;
//         let cur_end_dt = cur_total_cost_infos.end_dt;
        
//         let pre_total_cost = pre_total_cost_infos.total_cost;
//         let pre_consume_list = pre_total_cost_infos.consume_list();
//         let pre_start_dt = pre_total_cost_infos.start_dt;
//         let pre_end_dt = pre_total_cost_infos.end_dt;
        

//         // Hand over the consumption details to Telegram bot.
//         send_message_consume_split(&self.bot, 
//             self.message_id, 
//             cur_consume_list, 
//             cur_total_cost, 
//             cur_start_dt, 
//             cur_end_dt,
//             cur_empty_flag
//         ).await?;  
        
        
//         // ( consumption type information, consumption type graph storage path )
//         let comsume_type_infos = get_consume_type_graph(cur_total_cost, cur_start_dt, cur_end_dt, cur_consume_list).await?;
//         let consume_type_list = &comsume_type_infos.0;
//         let consume_type_img = comsume_type_infos.1;

//         let mut python_graph_line_info_cur = ToPythonGraphLine::new(
//             "cur", 
//             get_str_from_naivedate(cur_start_dt).as_str(), 
//             get_str_from_naivedate(cur_end_dt).as_str(), 
//             cur_total_cost, 
//             cur_consume_list.clone())?;
        
//         let mut python_graph_line_info_pre = ToPythonGraphLine::new(
//         "pre", 
//         get_str_from_naivedate(pre_start_dt).as_str(), 
//         get_str_from_naivedate(pre_end_dt).as_str(), 
//         pre_total_cost, 
//         pre_consume_list.clone())?;
        
//         let graph_path = get_consume_detail_graph_double(&mut python_graph_line_info_cur, &mut python_graph_line_info_pre).await?;


//         send_photo_confirm(&self.bot, self.message_id, &graph_path).await?;
//         send_photo_confirm(&self.bot, self.message_id, &consume_type_img).await?;

//         send_message_consume_type(&self.bot, 
//                     self.message_id, 
//                             consume_type_list, 
//                             cur_total_cost, 
//                             cur_start_dt, 
//                             cur_end_dt,
//                             cur_empty_flag).await?;  
        
//         let delete_target_vec: Vec<String> = vec![consume_type_img, graph_path];
//         delete_file(delete_target_vec)?;

//         Ok(())
//     }



//     /*
//         command handler: Writes the expenditure details to the index in ElasticSearch. -> c
//     */
//     pub async fn command_consumption(&self) -> Result<(), anyhow::Error> {

//         let args = &self.input_text[2..];
            
//         let split_args_vec: Vec<String> = args.split(':').map(|s| s.to_string()).collect();
//         let mut consume_name = "";
//         let mut consume_cash = "";
        
//         if split_args_vec.len() != 2 {
            
//             send_message_confirm(&self.bot, 
//                                 self.message_id, 
//                                 "There is a problem with the parameter you entered. Please check again. \nEX) c snack:15000").await?;

//             return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", self.input_text)));
//         } 
        
//         if let Some(cons_name) = split_args_vec.get(0) {

//             if let Some(price) = split_args_vec.get(1) {

//                 if !is_numeric(price) {
//                     send_message_confirm(&self.bot, self.message_id, "The second parameter must be numeric. \nEX) c snack:15000").await?;
//                     return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", self.input_text)));
//                 }

//                 consume_name = cons_name;
//                 consume_cash = price;
//             }        

//         } else {
            
//             send_message_confirm(&self.bot, 
//                             self.message_id, 
//                             "There is a problem with the parameter you entered. Please check again. \nEX) c snack:15000").await?;

//             return Err(anyhow!(format!("[Parameter Error][command_consumption()] Invalid format of 'text' variable entered as parameter. : {:?}", text)));
//         }
        
//         let curr_time = get_current_kor_naive_datetime();
        
//         let document = json!({
//             "@timestamp": get_str_from_naive_datetime(curr_time),
//             "prodt_name": consume_name,
//             "prodt_money": convert_numeric(consume_cash)
//         });
        
//         let es_client = get_elastic_conn()?;
//         es_client.post_query(&document, "consuming_index_prod_new").await?;
        
//         Ok(())
//     }


//     /*
//         command handler: Checks how much you have consumed during a month -> cm
//     */
//     pub async fn command_consumption_per_mon(&self, text: &str) -> Result<(), anyhow::Error> {

//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();
        
//         let (cur_date_start, cur_date_end, one_mon_ago_date_start, one_mon_ago_date_end) = match split_args_vec.len() {
//             1 => {
//                 let start = get_current_kor_naivedate_first_date()?;
//                 let end = get_lastday_naivedate(start)?;
//                 let one_month_ago_start = get_add_month_from_naivedate(start, -1)?;
//                 let one_month_ago_end = get_lastday_naivedate(one_month_ago_start)?;
//                 (start, end, one_month_ago_start, one_month_ago_end)
//             },
//             2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) => {
                
//                 let year: i32 = split_args_vec
//                                     .get(0)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_mon()] The 0th data of 'split_args_vec' vector does not exist."))?
//                                     .parse()?;
                
//                 let month: u32 = split_args_vec
//                                     .get(1)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_mon()] The 1th data of 'split_args_vec' vector does not exist."))?
//                                     .parse()?;
                
//                 let start = get_naivedate(year, month, 1)?;
//                 let end = get_lastday_naivedate(start)?;
//                 let one_month_ago_start = get_add_month_from_naivedate(start, -1)?;
//                 let one_month_ago_end = get_lastday_naivedate(one_month_ago_start)?;
//                 (start, end, one_month_ago_start, one_month_ago_end)
//             },
//             _ => {
//                 send_message_confirm(&self.bot, self.message_id, "Invalid date format. Please use format YYYY.MM like cm 2023.07").await?;
//                 return Err(anyhow!("[Parameter Error][command_consumption_per_mon()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         };
        
//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_consumption_per_mon()] Cannot connect Elasticsearch"))?;

//         let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
//         let cur_mon_total_cost_infos = total_cost_detail_specific_period(cur_date_start, 
//                                                                                                 cur_date_end, 
//                                                                                                 es_client, 
//                                                                                                 "consuming_index_prod_new", 
//                                                                                                 &consume_type_vec).await?;
        
        
//         let pre_mon_total_cost_infos = total_cost_detail_specific_period(one_mon_ago_date_start, 
//                                                                                                 one_mon_ago_date_end, es_client, 
//                                                                                                 "consuming_index_prod_new", 
//                                                                                                 &consume_type_vec).await?;

//         self.command_common_double(cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;
        
//         Ok(())

//     }


//     /*
//         command handler: Checks how much you have consumed during a specific periods -> ctr
//     */
//     pub async fn command_consumption_per_term(&self, text: &str) -> Result<(), anyhow::Error> {

//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();
        
//         let (date_start, date_end, pre_date_start, pre_date_end) = match split_args_vec.len() {
            
//             2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}$").unwrap_or(false)) => {

//                 let split_bar_vec: Vec<String> = split_args_vec
//                                     .get(1)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_term()] The 1th data of 'split_args_vec' vector does not exist."))?
//                                     .split('-')
//                                     .map(String::from)
//                                     .collect();
                
//                 let date_start: String = split_bar_vec
//                                         .get(0)
//                                         .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_term()] The 0th data of 'split_bar_vec' vector does not exist."))?
//                                         .parse()?;
//                 let date_start_form = get_naive_date_from_str(&date_start, "%Y.%m.%d")?;

//                 let date_end: String = split_bar_vec
//                                     .get(1)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_term()] The 1th data of 'split_bar_vec' vector does not exist."))?
//                                     .parse()?;
                
//                 let date_end_form = get_naive_date_from_str(&date_end, "%Y.%m.%d")?;
                
//                 let pre_date_start = get_add_month_from_naivedate(date_start_form, -1)?;
//                 let pre_date_end = get_add_month_from_naivedate(date_end_form, -1)?;

//                 (date_start_form, date_end_form, pre_date_start, pre_date_end)
//             },
//             _ => {
//                 send_message_confirm(&self.bot, self.message_id, "There is a problem with the parameter you entered. Please check again. \nEX) ctr 2023.07.07-2023.08.01").await?;
//                 return Err(anyhow!("[Parameter Error][command_consumption_per_term()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         };
        
//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_consumption_per_term()] Cannot connect Elasticsearch"))?;
        
//         let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
            
//         let cur_mon_total_cost_infos = total_cost_detail_specific_period(date_start, 
//                                                                                                         date_end, 
//                                                                                                         es_client, 
//                                                                                                         "consuming_index_prod_new", 
//                                                                                                         &consume_type_vec).await?;
        
//         let pre_mon_total_cost_infos = total_cost_detail_specific_period(pre_date_start, 
//                                                                                                         pre_date_end, 
//                                                                                                         es_client, 
//                                                                                                         "consuming_index_prod_new", 
//                                                                                                         &consume_type_vec).await?;
                                                                                                        
//         self.command_common_double(cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;    
        
//         Ok(())

//     }



//     /*
//         command handler: Checks how much you have consumed during a day -> ct
//     */
//     pub async fn command_consumption_per_day(&self, message: &Message, text: &str, bot: &Bot) -> Result<(), anyhow::Error> {
        
//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();
        
//         let (start_dt, end_dt) = match split_args_vec.len() {
//             1 => {
//                 let start = get_current_kor_naivedate();
//                 let end = get_current_kor_naivedate();
//                 (start, end)
//             },
//             2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}$").unwrap_or(false)) => {
                
//                 let year: i32 = split_args_vec
//                                     .get(0)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_day()] The 0th data of 'split_args_vec' vector does not exist."))?
//                                     .parse()?;
                
//                 let month: u32 = split_args_vec
//                                     .get(1)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_day()] The 1th data of 'split_args_vec' vector does not exist."))?
//                                     .parse()?;

//                 let day: u32 = split_args_vec
//                     .get(2)
//                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_day()] The 2nd data of 'split_args_vec' vector does not exist."))?
//                     .parse()?;
                
//                 let start_dt = get_naivedate(year, month, day)?;
//                 let end_dt = get_naivedate(year, month, day)?;
                
//                 (start_dt, end_dt)
//             },
//             _ => {
//                 send_message_confirm(bot, message.chat.id, "There is a problem with the parameter you entered. Please check again. \nEX) ct or ct 2023.11.11").await?;
//                 return Err(anyhow!("[Parameter Error][command_consumption_per_day()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         };
        
//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_consumption_per_day()] Cannot connect Elasticsearch"))?;

//         let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?; 

//         let cur_mon_total_cost_infos = total_cost_detail_specific_period(start_dt, 
//                                                         end_dt, 
//                                                         es_client, 
//                                                         "consuming_index_prod_new", 
//                                                         &consume_type_vec).await?;
        
//         self.command_common_single(cur_mon_total_cost_infos).await?;

//         Ok(())
//     }


//     /*
//         command handler: Check the consumption details from the date of payment to the next payment. -> cs
//     */
//     pub async fn command_consumption_per_salary(&self, text: &str) -> Result<(), anyhow::Error> {

//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();
        
//         let (cur_date_start, cur_date_end, one_mon_ago_date_start, one_mon_ago_date_end) = match split_args_vec.len() {
            
//             1 => {
//                 let cur_day = get_current_kor_naivedate();
//                 let cur_year = cur_day.year();
//                 let cur_month = cur_day.month();
//                 let cur_date = cur_day.day();
                
//                 let cur_date_start  = if cur_date < 25 { 
//                     let date = get_naivedate(cur_year, cur_month, 25)?;
//                     get_add_month_from_naivedate(date, -1)?
//                 } else { 
//                     get_naivedate(cur_year, cur_month, 25)?
//                 };
                
//                 let cur_date_end  = if cur_date < 25 { 
//                     get_naivedate(cur_year, cur_month, 25)?
//                 } else { 
//                     let date = get_naivedate(cur_year, cur_month, 25)?;
//                     get_add_month_from_naivedate(date, 1)?
//                 };
                
//                 let one_mon_ago_date_start = get_add_month_from_naivedate(cur_date_start, -1)?;
//                 let one_mon_ago_date_end = get_add_month_from_naivedate(cur_date_end, -1)?;
                
                
//                 (cur_date_start, cur_date_end, one_mon_ago_date_start, one_mon_ago_date_end)
//             },
//             2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) => {
                
//                 let year: i32 = split_args_vec
//                                     .get(0)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_salary()] The 0th data of 'split_args_vec' vector does not exist."))?
//                                     .parse()?;
                
//                 let month: u32 = split_args_vec
//                                     .get(1)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_salary()] The 1th data of 'split_args_vec' vector does not exist."))?
//                                     .parse()?;
                
//                 let cur_date_end = get_naivedate(year, month, 25)?;
//                 let cur_date_start = get_add_month_from_naivedate(cur_date_end, -1)?;
                
//                 let one_mon_ago_date_start = get_add_month_from_naivedate(cur_date_start, -1)?;
//                 let one_mon_ago_date_end = get_add_month_from_naivedate(cur_date_end, -1)?;

//                 (cur_date_start, cur_date_end, one_mon_ago_date_start, one_mon_ago_date_end)
//             },
//             _ => {
//                 send_message_confirm(bot, message.chat.id, "There is a problem with the parameter you entered. Please check again. \nEX) ct or ct 2023.11.11").await?;
//                 return Err(anyhow!("[Parameter Error][command_consumption_per_salary()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         };
        
//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_consumption_per_salary()] Cannot connect Elasticsearch"))?;

//         let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
//         let cur_mon_total_cost_infos = total_cost_detail_specific_period(cur_date_start, 
//                                                                                                 cur_date_end, 
//                                                                                                 es_client, 
//                                                                                                 "consuming_index_prod_new", 
//                                                                                                 &consume_type_vec).await?;
        
        
//         let pre_mon_total_cost_infos = total_cost_detail_specific_period(one_mon_ago_date_start, 
//                                                                                                 one_mon_ago_date_end, es_client, 
//                                                                                                 "consuming_index_prod_new", 
//                                                                                                 &consume_type_vec).await?;
        
//         self.command_common_double(cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;  
        
//         Ok(())
//     }



//     /*
//         command handler: Checks how much you have consumed during a week -> cw
//     */
//     pub async fn command_consumption_per_week(message: &Message, text: &str, bot: &Bot) -> Result<(), anyhow::Error> {

//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();
        
//         let (date_start, date_end, one_pre_week_start, one_pre_week_end) = match split_args_vec.len() {
            
//             1 =>{

//                 let now = get_current_kor_naive_datetime();
//                 let today = now.date();

//                 let weekday = today.weekday();

//                 let days_until_monday = Weekday::Mon.num_days_from_monday() as i64 - weekday.num_days_from_monday() as i64;
//                 let monday = today + chrono::Duration::days(days_until_monday);

//                 let date_start = monday + chrono::Duration::days(0);
//                 let date_end = monday + chrono::Duration::days(6);  
//                 let one_pre_week_start = date_start - chrono::Duration::days(7);
//                 let one_pre_week_end = date_end - chrono::Duration::days(7);

//                 (date_start, date_end, one_pre_week_start, one_pre_week_end)
//             },
//             _ => {
//                 send_message_confirm(bot, message.chat.id, "There is a problem with the parameter you entered. Please check again. \nEX) cw").await?;
//                 return Err(anyhow!("[Parameter Error][command_consumption_per_week()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         };

//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_consumption_per_week()] Cannot connect Elasticsearch"))?;

//         let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
//         let cur_mon_total_cost_infos = total_cost_detail_specific_period(date_start, 
//                                                                                                 date_end, 
//                                                                                                 es_client, 
//                                                                                                 "consuming_index_prod_new", 
//                                                                                                 &consume_type_vec).await?;
        
        
//         let pre_mon_total_cost_infos = total_cost_detail_specific_period(one_pre_week_start, 
//                                                                                                 one_pre_week_end, 
//                                                                                                 es_client, 
//                                                                                                 "consuming_index_prod_new", 
//                                                                                                 &consume_type_vec).await?;
        
//         command_common_double(bot, message, cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;  
        
//         Ok(())
//     }



//     /*
//         command handler: Function for recording meal times -> mc
//     */
//     pub async fn command_record_fasting_time(message: &Message, text: &str, bot: &Bot) -> Result<(), anyhow::Error> { 

//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();
        
//         let meal_time = match split_args_vec.len() {
//             1 => {
//                 get_current_kor_naive_datetime()
//             },
//             2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{2}\:\d{2}$").unwrap_or(false)) => {
                
//                 let split_bar_vec: Vec<String> = split_args_vec
//                                     .get(1)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_record_fasting_time()] The 1th data of 'split_bar_vec' vector does not exist."))?
//                                     .split(':')
//                                     .map(String::from)
//                                     .collect();

//                 let hour = match split_bar_vec.get(0)
//                                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_record_fasting_time()] The 1th data of 'split_bar_vec' vector does not exist."))?
//                                     .parse::<u32>() {
//                                         Ok(hour) => hour,
//                                         Err(e) => return Err(anyhow!("[Parsing Error][command_record_fasting_time()] There was a problem parsing the 'hour' variable. : {:?}", e))
//                                     };
                
//                 let min = match split_bar_vec.get(1)
//                                     .ok_or_else(|| anyhow!("[Invalid date ERROR][command_record_fasting_time()] There is a problem with the 'min' variable."))?
//                                     .parse::<u32>() {
//                                         Ok(min) => min,
//                                         Err(e) => return Err(anyhow!("[Parsing ERROR][command_record_fasting_time()] There was a problem parsing the 'hour' variable. : {:?}", e))
//                                     };
                
//                 let meal_time_cur = get_current_kor_naive_datetime();

//                 meal_time_cur.date().and_hms_opt(hour, min, 0)
//                     .ok_or_else(|| anyhow!("[Invalid date ERROR][command_record_fasting_time()] There was a problem parsing the 'meal_time' variable."))?

//             },
//             _ => {
//                 send_message_confirm(bot, message.chat.id, "There is a problem with the parameter you entered. Please check again. \nEX01) mc 22:30 \nEX02) mc").await?;
//                 return Err(anyhow!("[Parameter Error][command_record_fasting_time()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         };
        
//         let es_client = get_elastic_conn()?;

//         // Brings the data of the most recent meal time of today's meal time.
//         let current_date = get_str_from_naivedate(get_current_kor_naivedate());

        // let es_query = json!({
        //     "size": 1,
        //     "query": {
        //     "range": {
        //         "@timestamp": {
        //         "gte": &current_date,
        //         "lte": &current_date
        //         }
        //     }
        //     },
        //     "sort": [
        //     { "@timestamp": { "order": "desc" }}
        //     ]
        // });

//         let last_stamp: i64 = get_recent_mealtime_data_from_elastic(&es_client, "meal_check_index", "laststamp", es_query, 0).await?;

//         let es_doc = json!({
//             "@timestamp": get_str_from_naive_datetime(meal_time),
//             "laststamp": last_stamp + 1,
//             "alarminfo": 0
//         });
        
//         es_client.post_query(&es_doc, "meal_check_index").await?;
        
//         send_message_confirm(bot, 
//                         message.chat.id,  
//                         &format!("The [{}] meal was finished at [ {} ]", 
//                             last_stamp + 1, 
//                             get_str_from_naive_datetime(meal_time))).await?;
        
//         Ok(())
//     }


//     /*
//         command handler: Check the fasting time. -> mt
//     */
//     pub async fn command_check_fasting_time(message: &Message, text: &str, bot: &Bot) -> Result<(), anyhow::Error> {
        
//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();
        
//         let current_datetime = match split_args_vec.len() {
//             1 => {
//                 get_current_kor_naive_datetime()
//             },
//             _ => {
//                 send_message_confirm(bot, message.chat.id, "There is a problem with the parameter you entered. Please check again. \nEX) mt").await?;
//                 return Err(anyhow!("[Parameter Error][command_check_fasting_time()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         };
        
//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_check_fasting_time()] Cannot connect Elasticsearch"))?;

//         let es_query = json!({
//             "size": 1,
//             "sort": [
//             { "@timestamp": { "order": "desc" }}
//             ]
//         });
        
//         let get_datetime: String = get_recent_mealtime_data_from_elastic(es_client, 
//                 "meal_check_index", 
//                 "@timestamp", 
//                 es_query, 
//                 String::from("")).await?;
        
//         let final_mealtime = get_naive_datetime_from_str(&get_datetime, "%Y-%m-%dT%H:%M:%SZ")?;

//         let duration = current_datetime - final_mealtime;

//         let laps_day = duration.num_days();
//         let laps_hours = duration.num_hours();
//         let laps_min = duration.num_minutes();
        
//         send_message_confirm(bot, 
//             message.chat.id, 
//             &format!("It's been {} days and {} hours and {} minutes since I kept the current fasting time.", 
//                 laps_day, 
//                 laps_hours,
//                 laps_min)).await?;
        
//         Ok(())
//     }



//     /*
//         command handler: Delete the last fasting time data. -> md
//     */
//     pub async fn command_delete_fasting_time(message: &Message, text: &str, bot: &Bot) -> Result<(), anyhow::Error> {

//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();
        
//         match split_args_vec.len() {
//             1 => { },
//             _ => {
//                 send_message_confirm(bot, message.chat.id, "There is a problem with the parameter you entered. Please check again. \nEX) md").await?;
//                 return Err(anyhow!("[Parameter Error][command_delete_fasting_time()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         }

//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_delete_fasting_time()] Cannot connect Elasticsearch"))?;

//         let es_query = json!({
//             "size": 1,
//             "sort": [
//             { "@timestamp": { "order": "desc" }}
//             ]
//         });
        
//         let get_doc_id: String = get_recent_mealtime_data_from_elastic(es_client, 
//             "meal_check_index", 
//             "_id", 
//             es_query, 
//             String::from("")).await?;
        
//         es_client.delete_query(&get_doc_id, "meal_check_index").await?;
        
//         Ok(())
//     }


//     /*
//         command handler: Checks how much you have consumed during one year -> cy
//     */
//     pub async fn command_consumption_per_year(message: &Message, text: &str, bot: &Bot) -> Result<(), anyhow::Error> {

//         let args = &text[2..];
//         let split_args_vec: Vec<String> = args.split(' ').map(String::from).collect();

//         let (date_start, date_end, one_year_pre_date_start, one_year_pre_date_end) = match split_args_vec.len() {
//             1 => {
                
//                 let cur_year = get_current_kor_naivedate();

//                 let start_date = get_naivedate(cur_year.year(), 1, 1)?;  
//                 let end_date = get_naivedate(cur_year.year(), 12, 31)?;   
//                 let one_year_pre_date_start = get_naivedate(cur_year.year() - 1, 1, 1)?;  
//                 let one_year_pre_date_end = get_naivedate(cur_year.year() - 1, 12, 31)?;   
                
//                 (start_date, end_date, one_year_pre_date_start, one_year_pre_date_end)
//             },
//             2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}$").unwrap_or(false)) => {
                
//                 let year: i32 = split_args_vec
//                     .get(0)
//                     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_per_year()] The 0th data of 'split_bar_vec' vector does not exist."))?
//                     .parse()?;

//                 let start_date = get_naivedate(year, 1, 1)?;  
//                 let end_date = get_naivedate(year, 12, 31)?;  
//                 let one_year_pre_date_start = get_naivedate(year - 1, 1, 1)?;  
//                 let one_year_pre_date_end = get_naivedate(year - 1, 12, 31)?;
                
//                 (start_date, end_date, one_year_pre_date_start, one_year_pre_date_end)
//             },
//             _ => {
//                 send_message_confirm(bot, message.chat.id, "There is a problem with the parameter you entered. Please check again. \nEX01) cy\nEX02) cy 2023").await?;
//                 return Err(anyhow!("[Parameter Error][command_consumption_per_year()] Invalid format of 'text' variable entered as parameter. : {:?}", text));
//             }
//         };
            
//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_consumption_per_year()] Cannot connect Elasticsearch"))?;

//         let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
//         let cur_mon_total_cost_infos = total_cost_detail_specific_period(date_start, 
//                                                                                                 date_end, 
//                                                                                                 es_client, 
//                                                                                                 "consuming_index_prod_new", 
//                                                                                                 &consume_type_vec).await?;
        
        
//         let pre_mon_total_cost_infos = total_cost_detail_specific_period(one_year_pre_date_start, 
//                                                                                                 one_year_pre_date_end, 
//                                                                                                 es_client, 
//                                                                                                 "consuming_index_prod_new", 
//                                                                                                 &consume_type_vec).await?;
        
//         command_common_double(bot, message, cur_mon_total_cost_infos, pre_mon_total_cost_infos).await?;  
        
//         Ok(())
//     }


//     /*
//         command handler: Writes the expenditure details to the index in ElasticSearch.
//     */
//     pub async fn command_consumption_auto(message: &Message, text: &str, bot: &Bot) -> Result<(), anyhow::Error> {

//         let re = Regex::new(r"\[.*?\]\n?").unwrap();
//         let replcae_string = re.replace_all(&text, "").to_string();

//         let split_args_vec: Vec<String> = replcae_string.split('\n').map(|s| s.to_string()).collect();
        
//         let card_comp = split_args_vec
//             .get(0)
//             .ok_or_else(|| anyhow!("[Parameter Error][command_consumption_auto()] Invalid format of 'text' variable entered as parameter : {:?}", split_args_vec))?;

//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_consumption_auto()] Cannot connect Elasticsearch"))?;
        

//         if card_comp.contains("nh") {
            
            // let consume_price_vec: Vec<String> = split_args_vec
            //     .get(2)
            //     .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed. : {:?}", 2, split_args_vec))?
            //     .replace(",", "")
            //     .replace("원", "")
            //     .split(" ")
            //     .map(|s| s.to_string())
            //     .collect(); 
            
//             let consume_price = consume_price_vec
//                 .get(0)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed.", 0))?
//                 .parse::<i32>()?;
            
//             let consume_time_vec: Vec<String> = split_args_vec
//                 .get(3)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 3))?
//                 .split(" ")
//                 .map(|s| s.to_string())
//                 .collect();
            
//             let date_part: Vec<u32> = consume_time_vec
//                 .get(0)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 0))?
//                 .split("/")
//                 .map(|s| s.parse::<u32>())
//                 .collect::<Result<Vec<_>, _>>()?;
            
//             let time_part: Vec<u32> = consume_time_vec
//                 .get(1)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 1))?
//                 .split(":")
//                 .map(|s| s.parse::<u32>())
//                 .collect::<Result<Vec<_>, _>>()?;
            
//             let mon = date_part.get(0).ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'date_part' vector was accessed.", 0))?;
//             let day = date_part.get(1).ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()]] Invalid index '{:?}' of 'date_part' vector was accessed.", 1))?;
//             let hour = time_part.get(0).ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'time_part' vector was accessed.", 0))?;
//             let min = time_part.get(1).ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'time_part' vector was accessed.", 1))?;
            
//             let consume_date = get_this_year_naivedatetime(*mon, *day, *hour, *min)?;
            
//             let consume_name = split_args_vec
//                 .get(4)
//                 .unwrap()
//                 .trim();
            
//             let document = json!({
//                 "@timestamp": get_str_from_naive_datetime(consume_date),
//                 "prodt_name": consume_name,
//                 "prodt_money": consume_price
//             });
            
//             es_client.post_query(&document, "consuming_index_prod_new").await?;
            
//         } else if card_comp.contains("삼성") {

//             let consume_price_vec: Vec<String> = split_args_vec
//                 .get(1)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed. : {:?}", 2, split_args_vec))?
//                 .replace(",", "")
//                 .replace("원", "")
//                 .split(" ")
//                 .map(|s| s.to_string())
//                 .collect(); 
            
//             let consume_price = consume_price_vec
//                 .get(0)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed.", 0))?
//                 .parse::<i32>()?;
            
//             let consume_time_name_vec: Vec<String> = split_args_vec
//                 .get(2)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 3))?
//                 .split(" ")
//                 .map(|s| s.to_string())
//                 .collect();
            
//             let date_part: Vec<u32> = consume_time_name_vec
//                 .get(0)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 0))?
//                 .split("/")
//                 .map(|s| s.parse::<u32>())
//                 .collect::<Result<Vec<_>, _>>()?;
            
//             let time_part: Vec<u32> = consume_time_name_vec
//                 .get(1)
//                 .ok_or_else(|| anyhow!("[[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 1))?
//                 .split(":")
//                 .map(|s| s.parse::<u32>())
//                 .collect::<Result<Vec<_>, _>>()?;
            
//             let mon = date_part.get(0).ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'date_part' vector was accessed.", 0))?;
//             let day = date_part.get(1).ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'date_part' vector was accessed.", 1))?;
//             let hour = time_part.get(0).ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'time_part' vector was accessed.", 0))?;
//             let min = time_part.get(1).ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'time_part' vector was accessed.", 1))?;
            
//             let consume_date = get_this_year_naivedatetime(*mon, *day, *hour, *min)?;
//             let consume_name = consume_time_name_vec
//                 .get(2)
//                 .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_consumption_auto()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed. - command_consumption_auto()", 2))?
//                 .trim();
            
//             let document = json!({
//                 "@timestamp": get_str_from_naive_datetime(consume_date),
//                 "prodt_name": consume_name,
//                 "prodt_money": consume_price
//             });
            
//             es_client.post_query(&document, "consuming_index_prod_new").await?;
            
//         } else {

//             send_message_confirm(bot, 
//                     message.chat.id, 
//                     "There is a problem with the parameter you entered. Please check again.").await?;
            
//             return Err(anyhow!(format!("[Parameter Error][command_consumption_auto()] Invalid format of 'text' variable entered as parameter. : {:?}", text)));
//         }
        
//         Ok(())
//     }


//     /*
//         command handler: Function that shows consumption type lists -> list
//     */
//     pub async fn command_get_consume_type_list(message: &Message, text: &str, bot: &Bot) -> Result<(), anyhow::Error> {

//         let args = &text[4..];
//         let split_args_vec: Vec<String> = args.trim().split(':').map(String::from).collect();
        
//         let es_client = ELASTICSEARCH_CLIENT
//             .get()
//             .ok_or_else(|| anyhow!("[DB Connection Error][command_get_consume_type_list()] Cannot connect Elasticsearch"))?;

//         match args.len() {
//             0 => {
//                 let consume_type_list: Vec<String> = get_classification_type(es_client, "consuming_index_prod_type").await?;
                
//                 if consume_type_list.len() == 0 {
//                     send_message_consume_type_list(bot, message.chat.id, &consume_type_list, true).await?;
//                 } else {
//                     send_message_consume_type_list(bot, message.chat.id, &consume_type_list, false).await?;
//                 }
//             },
//             _ => {
                
//                 if split_args_vec.len() == 0 {
//                     send_message_confirm(bot, message.chat.id, "Please specify both 'keyword_type' and 'keyword.' EX) 식사:하남돼지집:2").await?;
//                 } else {

//                     let prodt_type_list: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;

//                     let input_keyword_type = split_args_vec
//                         .get(0)
//                         .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_get_consume_type_list()] The 0th data of 'split_args_vec' vector does not exist."))?;

//                     let input_keyword = split_args_vec
//                         .get(1)
//                         .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_get_consume_type_list()] The 1st data of 'split_args_vec' vector does not exist."))?;

//                     let bias_val = split_args_vec
//                         .get(2)
//                         .ok_or_else(|| anyhow!("[Index Out Of Range Error][command_get_consume_type_list()] The 2nd data of 'split_args_vec' vector does not exist."))?
//                         .parse::<i32>()?;    

//                     let keyword_exists = prodt_type_list.iter().any(|elem| {
//                         elem.keyword_type == *input_keyword_type && elem.prodt_detail_list.iter().any(|prodt_elem| prodt_elem.keyword == *input_keyword)
//                     });
                    
//                     if keyword_exists {
//                         send_message_confirm(bot, message.chat.id, "This is the type that already exists.").await?;
//                     } else {

//                         let document = json!({
//                             "keyword_type": input_keyword_type,
//                             "keyword": input_keyword,
//                             "bias_value": bias_val
//                         });

//                         es_client.post_query(&document, "consuming_index_prod_type").await?;
//                     }
//                 }        
//             }
//         }
        
//         Ok(())
//     }
    

// }