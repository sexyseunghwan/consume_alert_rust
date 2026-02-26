use crate::common::*;

use crate::models::consume_prodt_info;
use crate::utils_modules::time_utils::*;

use crate::models::agg_result_set::*;
use crate::models::consume_prodt_info::*;
use crate::models::consume_prodt_info_by_installment::*;
use crate::models::consume_result_by_type::*;
use crate::models::document_with_id::*;
use crate::models::per_datetime::*;
use crate::models::spent_detail::*;
use crate::models::spent_detail_by_installment::*;
use crate::models::to_python_graph_circle::*;

#[async_trait]
pub trait ProcessService {
    fn process_by_consume_filter(
        &self,
        split_args_vec: &Vec<String>,
        user_seq: i64,
    ) -> Result<SpentDetailByInstallment, anyhow::Error>;
    fn get_spent_detail_installment_process(
        &self,
        spent_detail_by_installment: &SpentDetailByInstallment,
    ) -> Result<Vec<SpentDetail>, anyhow::Error>;
    fn get_nmonth_to_current_date(
        &self,
        date_start: NaiveDate,
        date_end: NaiveDate,
        nmonth: i32,
    ) -> Result<PerDatetime, anyhow::Error>;
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

impl ProcessServicePub {
    #[doc = "Functions that vectorize by spaces, excluding certain characters from a string (internal helper)"]
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

    #[doc = "Function that parses the money spent (internal helper)"]
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

    #[doc = "Function that parses date data from consumption data (internal helper)"]
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

    #[doc = "Function that parses date data and returns DateTime<Local> (internal helper)"]
    /// # Arguments
    /// * `consume_time_name_vec` - Vector with date, time data : ex) ["11/25", "10:02"]
    ///
    /// # Returns
    /// * Result<DateTime<Local>, anyhow::Error>
    fn get_consume_datetime_local(
        &self,
        consume_time_name_vec: &Vec<String>,
    ) -> Result<DateTime<Local>, anyhow::Error> {
        /* "11/25" */
        let parsed_date: &String = consume_time_name_vec
            .get(0)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_datetime_local()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", 0))?;

        /* "10:02" */
        let parsed_time: &String = consume_time_name_vec
            .get(1)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_datetime_local()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", 1))?;

        let cur_year: i32 = get_current_kor_naivedate().year();
        let formatted_date_str: String = format!("{}/{}", cur_year, parsed_date);
        let format_date: NaiveDate = NaiveDate::parse_from_str(&formatted_date_str, "%Y/%m/%d")?;
        let format_time: NaiveTime = NaiveTime::parse_from_str(&parsed_time, "%H:%M")?;
        let format_naive_datetime: NaiveDateTime = NaiveDateTime::new(format_date, format_time);

        // Convert NaiveDateTime to DateTime<Local> using Seoul timezone
        let datetime_local: DateTime<Local> = Seoul
            .from_local_datetime(&format_naive_datetime)
            .single()
            .ok_or_else(|| {
                anyhow!(
                    "[Error][get_consume_datetime_local()] Failed to convert to DateTime<Local>"
                )
            })?
            .with_timezone(&Local);

        Ok(datetime_local)
    }

    #[doc = "Installment filtering function : string -> i64 (internal helper)"]
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

    #[doc = "Function that calculates the money spent by category (internal helper)"]
    /// # Arguments
    /// * `total_cost` - total money spent
    /// * `type_map` - <Consumption classification, money spent>
    ///
    /// # Returns
    /// * Result<Vec<ConsumeResultByType>, anyhow::Error>
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

    #[doc = "Process NH card payment data (internal helper function)"]
    /// # Arguments
    /// * `split_args_vec` - Payment information vector
    /// * `user_seq` - User sequence number
    ///
    /// # Returns
    /// * Result<SpentDetailByInstallment, anyhow::Error>
    fn process_nh_card(
        &self,
        split_args_vec: &Vec<String>,
        user_seq: i64,
    ) -> Result<SpentDetailByInstallment, anyhow::Error> {
        let split_val: Vec<&str> = vec![",", "원"];

        // Extract price information
        let price_str = split_args_vec
            .get(2)
            .ok_or_else(|| anyhow!("[NH Card] Price field (index 2) not found"))?;
        let consume_price_vec: Vec<String> =
            self.get_string_vector_by_replace(price_str, &split_val)?;
        let consume_price: i64 = self.get_consume_prodt_money(&consume_price_vec, 0)?;

        // Extract time information
        let time_str = split_args_vec
            .get(3)
            .ok_or_else(|| anyhow!("[NH Card] Time field (index 3) not found"))?;
        let consume_time_vec: Vec<String> =
            time_str.split(" ").map(|s| s.trim().to_string()).collect();
        let spent_at: DateTime<Local> = self.get_consume_datetime_local(&consume_time_vec)?;

        // Extract product name
        let consume_name: &String = split_args_vec
            .get(4)
            .ok_or_else(|| anyhow!("[NH Card] Product name field (index 4) not found"))?;

        let spent_detail: SpentDetail = SpentDetail::new(
            consume_name.clone(),
            consume_price,
            spent_at,
            1, // should_index = 1 (true)
            user_seq,
            1, // spent_group_id
            1,
        );

        Ok(SpentDetailByInstallment::new(0, spent_detail))
    }

    #[doc = "Process Samsung card payment data (internal helper function)"]
    /// # Arguments
    /// * `split_args_vec` - Payment information vector
    /// * `user_seq` - User sequence number
    ///
    /// # Returns
    /// * Result<SpentDetailByInstallment, anyhow::Error>
    fn process_samsung_card(
        &self,
        split_args_vec: &Vec<String>,
        user_seq: i64,
    ) -> Result<SpentDetailByInstallment, anyhow::Error> {
        let split_val: Vec<&str> = vec![",", "원"];

        // Extract price and payment type
        let price_str = split_args_vec
            .get(1)
            .ok_or_else(|| anyhow!("[Samsung Card] Price field (index 1) not found"))?;
        let consume_price_vec: Vec<String> =
            self.get_string_vector_by_replace(price_str, &split_val)?;
        let consume_price: i64 = self.get_consume_prodt_money(&consume_price_vec, 0)?;

        let payment_type: &str = consume_price_vec
            .get(1)
            .ok_or_else(|| anyhow!("[Samsung Card] Payment type not found"))?
            .trim();
        let monthly_installment_plan: i64 = self.get_installment_payment_filtering(payment_type)?;

        // Extract time and product name
        let time_str = split_args_vec
            .get(2)
            .ok_or_else(|| anyhow!("[Samsung Card] Time field (index 2) not found"))?;
        let consume_time_vec: Vec<String> = time_str.split(" ").map(|s| s.to_string()).collect();
        let spent_at: DateTime<Local> = self.get_consume_datetime_local(&consume_time_vec)?;

        let consume_name: &String = consume_time_vec
            .get(2)
            .ok_or_else(|| anyhow!("[Samsung Card] Product name not found in time field"))?;

        let spent_detail: SpentDetail = SpentDetail::new(
            consume_name.clone(),
            consume_price,
            spent_at,
            1, // should_index = 1 (true)
            user_seq,
            1, // spent_group_id
            1,
        );

        Ok(SpentDetailByInstallment::new(
            monthly_installment_plan,
            spent_detail,
        ))
    }

    #[doc = "Process Shinhan card payment data (internal helper function)"]
    /// # Arguments
    /// * `split_args_vec` - Payment information vector
    /// * `user_seq` - User sequence number
    ///
    /// # Returns
    /// * Result<SpentDetailByInstallment, anyhow::Error>
    fn process_shinhan_card(
        &self,
        split_args_vec: &Vec<String>,
        user_seq: i64,
    ) -> Result<SpentDetailByInstallment, anyhow::Error> {
        let split_val: Vec<&str> = vec![",", "원"];

        // Extract and parse price information
        let first_field = split_args_vec
            .get(0)
            .ok_or_else(|| anyhow!("[Shinhan Card] First field (index 0) not found"))?;
        let consume_price_vec: Vec<String> =
            self.get_string_vector_by_replace(first_field, &split_val)?;

        let spent_detail: &String = consume_price_vec
            .get(2)
            .ok_or_else(|| anyhow!("[Shinhan Card] Price and date field not found"))?;

        // Parse format: "123456(일시불)123"
        let split_by_front: Vec<String> = spent_detail
            .split("(")
            .map(|s| s.trim().to_string())
            .collect();
        let split_by_back: Vec<String> = spent_detail
            .split(")")
            .map(|s| s.trim().to_string())
            .collect();

        let consume_price: i64 = split_by_front
            .get(0)
            .ok_or_else(|| anyhow!("[Shinhan Card] Price parsing failed"))?
            .parse::<i64>()?;

        let payment_type: String = split_by_front
            .get(1)
            .ok_or_else(|| anyhow!("[Shinhan Card] Payment type parsing failed"))?
            .replace(")", "");

        let monthly_installment_plan: i64 =
            self.get_installment_payment_filtering(&payment_type)?;

        // Extract date and time
        let consume_date: String = split_by_back
            .get(1)
            .ok_or_else(|| anyhow!("[Shinhan Card] Date parsing failed"))?
            .to_string();

        let consume_time: String = consume_price_vec
            .get(3)
            .ok_or_else(|| anyhow!("[Shinhan Card] Time field not found"))?
            .to_string();

        let consume_time_vec: Vec<String> = vec![consume_date, consume_time];
        let spent_at: DateTime<Local> = self.get_consume_datetime_local(&consume_time_vec)?;

        // Extract product name
        let consume_name = consume_price_vec
            .get(4)
            .ok_or_else(|| anyhow!("[Shinhan Card] Product name not found"))?;

        let spent_detail: SpentDetail = SpentDetail::new(
            consume_name.clone(),
            consume_price,
            spent_at,
            1, // should_index = 1 (true)
            user_seq,
            1, // spent_group_id
            1,
        );

        Ok(SpentDetailByInstallment::new(
            monthly_installment_plan,
            spent_detail,
        ))
    }
}

#[async_trait]
impl ProcessService for ProcessServicePub {
    #[doc = "Process processing function based on the type of payment"]
    /// # Arguments
    /// * `split_args_vec` - Array with strings as elements : Payment-related information vector:
    /// - ex) ["nh카드3*3*승인", "신*환", "5,500원 일시불", "11/25 10:02", "메가엠지씨커피 선릉", "총누적469,743원"]
    /// * `user_seq` - User sequence number
    ///
    /// # Returns
    /// * Result<SpentDetailByInstallment, anyhow::Error>
    fn process_by_consume_filter(
        &self,
        split_args_vec: &Vec<String>,
        user_seq: i64,
    ) -> Result<SpentDetailByInstallment, anyhow::Error> {
        let consume_type: &String = split_args_vec
            .get(0)
            .ok_or_else(|| anyhow!("[Parameter Error][process_by_consume_filter] Invalid format of 'text' variable entered as parameter : {:?}", split_args_vec))?;

        if consume_type.contains("nh") {
            self.process_nh_card(split_args_vec, user_seq)
        } else if consume_type.contains("삼성") {
            self.process_samsung_card(split_args_vec, user_seq)
        } else if consume_type.contains("신한카드") {
            self.process_shinhan_card(split_args_vec, user_seq)
        } else {
            Err(anyhow!("[Error][process_by_consume_filter] Variable 'consume_type' contains an undefined string: {}", consume_type))
        }
    }

    #[doc = "Functions that take into account installment payments"]
    /// # Arguments
    /// * `spent_detail_by_installment` - Spent detail with installment information
    ///
    /// # Returns
    /// * Result<Vec<SpentDetail>, anyhow::Error>
    fn get_spent_detail_installment_process(
        &self,
        spent_detail_by_installment: &SpentDetailByInstallment,
    ) -> Result<Vec<SpentDetail>, anyhow::Error> {
        let spent_detail: &SpentDetail = spent_detail_by_installment.spent_detail();
        let mut spent_detail_vec: Vec<SpentDetail> = Vec::new();

        if *spent_detail_by_installment.installment() > 0 {
            let spent_money: i64 = *spent_detail.spent_money();
            let spent_money_ceil: i64 = (spent_money as f64
                / *spent_detail_by_installment.installment() as f64)
                .ceil() as i64;

            for idx in 0..*spent_detail_by_installment.installment() {
                let mut spent_detail_clone: SpentDetail = spent_detail.clone();

                let spent_at: DateTime<Local> = *spent_detail_clone.spent_at();
                let calculate_spent_at: DateTime<Local> =
                    spent_at + chrono::Duration::days(30 * (idx as i64));

                spent_detail_clone.set_spent_at(calculate_spent_at);
                spent_detail_clone.set_spent_money(spent_money_ceil);
                spent_detail_clone.set_spent_name(format!(
                    "{}-{}/{}",
                    spent_detail.spent_name(),
                    idx + 1,
                    spent_detail_by_installment.installment()
                ));

                spent_detail_vec.push(spent_detail_clone);
            }
        } else {
            spent_detail_vec.push(spent_detail.clone());
        }

        Ok(spent_detail_vec)
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

        let mut cost_map: HashMap<String, i64> =
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

        cost_map.retain(|_, v| *v >= 0);

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
