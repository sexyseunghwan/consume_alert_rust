use crate::common::*;

use crate::models::consume_prodt_info;
use crate::utils_modules::time_utils::*;

use crate::models::agg_result_set::*;
use crate::models::consume_prodt_info::*;
use crate::models::consume_prodt_info_by_installment::*;
use crate::models::consume_result_by_type::*;
use crate::models::document_with_id::*;
use crate::models::per_datetime::*;
use crate::models::to_python_graph_circle::*;

#[async_trait]
pub trait ProcessService {
    fn get_string_vector_by_replace(
        &self,
        intput_str: &str,
        replacements: &Vec<&str>,
    ) -> Result<Vec<String>, anyhow::Error>;
    fn get_consume_prodt_money(
        &self,
        consume_price_vec: &Vec<String>,
        idx: usize,
    ) -> Result<i64, anyhow::Error>;
    fn get_consume_time(
        &self,
        consume_time_name_vec: &Vec<String>,
    ) -> Result<String, anyhow::Error>;
    fn get_installment_payment_filtering(&self, payment_type: &str) -> Result<i64, anyhow::Error>;
    fn process_by_consume_filter(
        &self,
        split_args_vec: &Vec<String>,
    ) -> Result<ConsumeProdtInfoByInstallment, anyhow::Error>;
    fn get_consume_prodt_info_installment_process(
        &self,
        consume_prodt_info_by_installment: &ConsumeProdtInfoByInstallment,
    ) -> Result<Vec<ConsumeProdtInfo>, anyhow::Error>;
    fn get_nmonth_to_current_date(
        &self,
        date_start: NaiveDate,
        date_end: NaiveDate,
        nmonth: i32,
    ) -> Result<PerDatetime, anyhow::Error>;
    fn get_calculate_pie_infos_from_category(
        &self,
        total_cost: f64,
        type_map: &HashMap<String, i64>,
    ) -> Result<Vec<ConsumeResultByType>, anyhow::Error>;
    fn convert_consume_result_by_type_to_python_graph_circle(
        &self,
        consume_result_by_types: &Vec<ConsumeResultByType>,
        total_cost: f64,
        start_dt: NaiveDate,
        end_dt: NaiveDate,
    ) -> Result<ToPythonGraphCircle, anyhow::Error>;
    fn get_consumption_result_by_category(
        &self,
        consume_details: &AggResultSet<ConsumeProdtInfo>,
    ) -> Result<Vec<ConsumeResultByType>, anyhow::Error>;
    fn get_nday_to_current_date(
        &self,
        date_start: NaiveDate,
        date_end: NaiveDate,
        nday: i32,
    ) -> Result<PerDatetime, anyhow::Error>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct ProcessServicePub;

#[async_trait]
impl ProcessService for ProcessServicePub {
    #[doc = "Functions that vectorize by spaces, excluding certain characters from a string"]
    /// # Arguments
    /// * `intput_str`  - Applied String : ex) "289,545원 일시불"
    /// * `replacements`- Character vector to replace : ex) [",", "원"]
    ///
    /// # Returns
    /// * Result<Vec<String>, anyhow::Error> - ex) ["289545", "일시불"]
    fn get_string_vector_by_replace(
        &self,
        intput_str: &str,
        replacements: &Vec<&str>,
    ) -> Result<Vec<String>, anyhow::Error> {
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

    #[doc = "Function that parses the money spent"]
    /// # Arguments
    /// * `consume_price_vec`  - Vector with money spent data
    /// * `idx`- Index of the vector to be accessed
    ///
    /// # Returns
    /// * Result<i64, anyhow::Error
    fn get_consume_prodt_money(
        &self,
        consume_price_vec: &Vec<String>,
        idx: usize,
    ) -> Result<i64, anyhow::Error> {
        let consume_price: i64 = consume_price_vec
            .get(idx)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_prodt_money()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed.", idx))?
            .parse::<i64>()?;

        Ok(consume_price)
    }

    #[doc = "Function that parses date data from consumption data"]
    /// # Arguments
    /// * `consume_time_name_vec` - Vector with date, time data : ex) ["11/25", "10:02"]
    ///
    /// # Returns
    /// * Result<String, anyhow::Error>
    fn get_consume_time(
        &self,
        consume_time_name_vec: &Vec<String>,
    ) -> Result<String, anyhow::Error> {
        /* "11/25" */
        let parsed_date: &String = consume_time_name_vec
            .get(0)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_time()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", 0))?;

        /* "10:02" */
        let parsed_time: &String = consume_time_name_vec
            .get(1)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_time()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", 1))?;

        let cur_year: i32 = get_current_kor_naivedate().year();
        let formatted_date_str: String = format!("{}/{}", cur_year, parsed_date);
        let format_date: NaiveDate = NaiveDate::parse_from_str(&formatted_date_str, "%Y/%m/%d")?;
        let format_time: NaiveTime = NaiveTime::parse_from_str(&parsed_time, "%H:%M")?;
        let format_datetime: NaiveDateTime = NaiveDateTime::new(format_date, format_time);

        Ok(format_datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    }

    #[doc = "Installment filtering function : string -> i64"]
    /// # Arguments
    /// * `payment_type` - Lump sum or installment payment type
    ///
    /// # Returns
    /// * Result<i64, anyhow::Error>
    fn get_installment_payment_filtering(&self, payment_type: &str) -> Result<i64, anyhow::Error> {
        let installment_payment: i64 = match payment_type {
            "일시불" => 0,
            "03개월" => 3,
            "06개월" => 6,
            "09개월" => 9,
            "12개월" => 12,
            _ => 0,
        };

        Ok(installment_payment)
    }

    #[doc = "Process processing function based on the type of payment"]
    /// # Arguments
    /// * `split_args_vec` - Array with strings as elements : Payment-related information vector:
    /// - ex) ["nh카드3*3*승인", "신*환", "5,500원 일시불", "11/25 10:02", "메가엠지씨커피 선릉", "총누적469,743원"]
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    fn process_by_consume_filter(
        &self,
        split_args_vec: &Vec<String>,
    ) -> Result<ConsumeProdtInfoByInstallment, anyhow::Error> {
        let consume_type: &String = split_args_vec
            .get(0)
            .ok_or_else(|| anyhow!("[Parameter Error][process_by_consume_type()] Invalid format of 'text' variable entered as parameter : {:?}", split_args_vec))?;

        let cur_timestamp: String = get_str_curdatetime();
        let split_val: Vec<&str> = vec![",", "원"];

        if consume_type.contains("nh") {
            let consume_price_vec: Vec<String> = self.get_string_vector_by_replace(split_args_vec
                .get(2)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed. : {:?}", 2, split_args_vec))?,
                &split_val
            )?;

            let consume_price: i64 = self.get_consume_prodt_money(&consume_price_vec, 0)?;

            let consume_time_vec: Vec<String> = split_args_vec
                .get(3)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 3))?
                .split(" ")
                .map(|s| s.trim().to_string())
                .collect();

            let consume_time: String = self.get_consume_time(&consume_time_vec)?;

            let consume_name: &String = split_args_vec
                .get(4)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'split_args_vec' vector was accessed.", 4))?;

            let res_struct: ConsumeProdtInfo = ConsumeProdtInfo::new(
                consume_time,
                cur_timestamp,
                consume_name.clone(),
                consume_price,
                String::from("etc"),
            );

            let consume_prodt_info_by_installment: ConsumeProdtInfoByInstallment =
                ConsumeProdtInfoByInstallment::new(0, res_struct);

            Ok(consume_prodt_info_by_installment)
        } else if consume_type.contains("삼성") {
            let consume_price_vec: Vec<String> = self.get_string_vector_by_replace(split_args_vec
                .get(1)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed. : {:?}", 1, split_args_vec))?,
                &split_val
            )?;

            let consume_price: i64 = self.get_consume_prodt_money(&consume_price_vec, 0)?;

            let payment_type: &str = consume_price_vec
                .get(1)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'split_args_vec' vector was accessed.", 1))?
                .trim();

            /* It determines whether it is an 'installment payment' or a 'lump sum payment.' */
            let monthly_installment_plan: i64 =
                self.get_installment_payment_filtering(payment_type)?;

            let consume_time_vec: Vec<String> = split_args_vec
                .get(2)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 2))?
                .split(" ")
                .map(|s| s.to_string())
                .collect();

            let consume_time: String = self.get_consume_time(&consume_time_vec)?;

            let consume_name: &String = consume_time_vec
                .get(2)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_time_vec' vector was accessed.", 2))?;

            let res_struct: ConsumeProdtInfo = ConsumeProdtInfo::new(
                consume_time,
                cur_timestamp,
                consume_name.clone(),
                consume_price,
                String::from("etc"),
            );

            let consume_prodt_info_by_installment: ConsumeProdtInfoByInstallment =
                ConsumeProdtInfoByInstallment::new(monthly_installment_plan, res_struct);

            Ok(consume_prodt_info_by_installment)
        } else {
            return Err(anyhow!("[Error][process_by_consume_type()] Variable 'consume_type' contains an undefined string."));
        }
    }

    #[doc = "Functions that take into account installment payments"]
    /// # Arguments
    /// * `consume_prodt_info` - Consumption information
    ///
    /// # Returns
    /// * Result<Vec<ConsumeProdtInfo>, anyhow::Error>
    fn get_consume_prodt_info_installment_process(
        &self,
        consume_prodt_info_by_installment: &ConsumeProdtInfoByInstallment,
    ) -> Result<Vec<ConsumeProdtInfo>, anyhow::Error> {
        let consume_prodt_info: &ConsumeProdtInfo =
            consume_prodt_info_by_installment.consume_prodt_info();
        let mut consume_prodt_info_vec: Vec<ConsumeProdtInfo> = Vec::new();

        if *consume_prodt_info_by_installment.installment() > 0 {
            let prodt_money: i64 = consume_prodt_info.prodt_money;
            let prodt_money_ceil: i64 = (prodt_money as f64
                / consume_prodt_info_by_installment.installment as f64)
                .ceil() as i64;

            for idx in 0..consume_prodt_info_by_installment.installment {
                let mut consume_prodt_info_clone: ConsumeProdtInfo = consume_prodt_info.clone();

                let timestamp: NaiveDateTime = get_naive_datetime_from_str(
                    consume_prodt_info_clone.timestamp(),
                    "%Y-%m-%dT%H:%M:%SZ",
                )?;
                let calculate_timestamp: NaiveDateTime =
                    timestamp + chrono::Duration::days(30 * (idx as i64));

                consume_prodt_info_clone.set_timestamp(calculate_timestamp.to_string());
                consume_prodt_info_clone.set_prodt_money(prodt_money_ceil);
                consume_prodt_info_clone.set_prodt_name(format!(
                    "{}-{}/{}",
                    consume_prodt_info.prodt_name(),
                    idx + 1,
                    consume_prodt_info_by_installment.installment
                ));

                consume_prodt_info_vec.push(consume_prodt_info_clone);
            }
        } else {
            consume_prodt_info_vec.push(consume_prodt_info.clone());
        }

        Ok(consume_prodt_info_vec)
    }

    #[doc = "Function that returns the time allotted as a parameter and the time before/after `N` months"]
    /// # Arguments
    /// * `date_start` - Start date
    /// * `date_end` - End date    
    /// * `nmonth` - Before or after `N` months
    ///
    /// # Returns
    /// * Result<PermonDatetime, anyhow::Error>  
    fn get_nmonth_to_current_date(
        &self,
        date_start: NaiveDate,
        date_end: NaiveDate,
        nmonth: i32,
    ) -> Result<PerDatetime, anyhow::Error> {
        let n_month_start: NaiveDate = get_add_month_from_naivedate(date_start, nmonth)
            .map_err(|e| anyhow!("{:?} -> in get_nmonth_to_current_date().n_month_start", e))?;

        let n_month_end: NaiveDate = get_add_month_from_naivedate(date_end, nmonth)
            .map_err(|e| anyhow!("{:?} -> in get_nmonth_to_current_date().n_month_end", e))?;

        let per_mon_datetim: PerDatetime =
            PerDatetime::new(date_start, date_end, n_month_start, n_month_end);

        Ok(per_mon_datetim)
    }

    #[doc = "Function that calculates the money spent by category"]
    /// # Arguments
    /// * `total_mount` - total money spent
    /// * `type_map` - <Consumption classification, money spent>
    ///
    /// # Returns
    /// * Result<HashMap<String, i64>, anyhow::Error>
    fn get_calculate_pie_infos_from_category(
        &self,
        total_cost: f64,
        type_map: &HashMap<String, i64>,
    ) -> Result<Vec<ConsumeResultByType>, anyhow::Error> {
        let consume_result_by_types : Vec<ConsumeResultByType> = type_map
            .iter()
            .map(|(key, value)| {
                let prodt_type: String = key.to_string();
                let prodt_cost: i64 = *value;

                let prodt_per: f64 = (prodt_cost as f64 / total_cost as f64) * 100.0;
                let prodt_per_rounded: f64 = (prodt_per * 10.0).round() / 10.0; /* Round to the second decimal place */

                ConsumeResultByType::new(prodt_type, prodt_cost, prodt_per_rounded)
            })
            .collect();

        Ok(consume_result_by_types)
    }

    #[doc = "Function that converts consumption results by category into Python data"]
    /// # Arguments
    /// * `consume_details` - Consumption details
    ///
    /// # Returns
    /// * Result<Vec<ConsumeResultByType>, anyhow::Error>
    fn get_consumption_result_by_category(
        &self,
        consume_details: &AggResultSet<ConsumeProdtInfo>,
    ) -> Result<Vec<ConsumeResultByType>, anyhow::Error> {
        let consume_inner_details: &Vec<DocumentWithId<ConsumeProdtInfo>> =
            consume_details.source_list();
        let total_cost: f64 = *consume_details.agg_result();

        let cost_map: HashMap<String, i64> =
            consume_inner_details
                .iter()
                .fold(HashMap::new(), |mut acc, consume_detail| {
                    let detail: &ConsumeProdtInfo = consume_detail.source();
                    let prodt_type: String = detail.prodt_type().to_string();
                    let prodt_money: i64 = *detail.prodt_money();

                    acc.entry(prodt_type)
                        .and_modify(|value| *value += prodt_money)
                        .or_insert(prodt_money);
                    acc
                });

        let mut consume_result_by_types: Vec<ConsumeResultByType> =
            self.get_calculate_pie_infos_from_category(total_cost, &cost_map)?;

        consume_result_by_types.sort_by(|a, b| {
            b.consume_prodt_cost
                .partial_cmp(&a.consume_prodt_cost)
                .unwrap_or(Ordering::Equal)
        });

        Ok(consume_result_by_types)
    }

    #[doc = "Vec<ConsumeResultByType> -> ToPythonGraphCircle"]
    /// # Arguments
    /// * `consume_result_by_types` - Consumption results by category
    ///
    /// # Returns
    /// * Result<ToPythonGraphCircle, anyhow::Error>
    fn convert_consume_result_by_type_to_python_graph_circle(
        &self,
        consume_result_by_types: &Vec<ConsumeResultByType>,
        total_cost: f64,
        start_dt: NaiveDate,
        end_dt: NaiveDate,
    ) -> Result<ToPythonGraphCircle, anyhow::Error> {
        let (prodt_type_vec, prodt_type_cost_per_vec): (Vec<String>, Vec<f64>) =
            consume_result_by_types
                .iter()
                .map(|elem| {
                    (
                        elem.consume_prodt_type().to_string(),
                        *elem.consume_prodt_per(),
                    )
                })
                .unzip();

        let to_python_graph_circle: ToPythonGraphCircle = ToPythonGraphCircle::new(
            prodt_type_vec,
            prodt_type_cost_per_vec,
            start_dt.to_string(),
            end_dt.to_string(),
            total_cost,
        );

        Ok(to_python_graph_circle)
    }

    #[doc = "Function that returns the time allotted as a parameter and the time before/after `N` days"]
    /// # Arguments
    /// * `date_start`  
    /// * `date_end`    
    /// * `nday` - Before or after `N` days
    ///
    /// # Returns
    /// * Result<PermonDatetime, anyhow::Error>
    fn get_nday_to_current_date(
        &self,
        date_start: NaiveDate,
        date_end: NaiveDate,
        nday: i32,
    ) -> Result<PerDatetime, anyhow::Error> {
        let n_day_start: NaiveDate = get_add_date_from_naivedate(date_start, nday)?;
        let n_day_end: NaiveDate = get_add_date_from_naivedate(date_end, nday)?;

        let per_day_datetim: PerDatetime =
            PerDatetime::new(date_start, date_end, n_day_start, n_day_end);

        Ok(per_day_datetim)
    }
}
