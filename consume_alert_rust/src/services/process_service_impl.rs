use crate::common::*;

use crate::utils_modules::time_utils::*;

use crate::models::agg_result_set::*;
use crate::models::consume_result_by_type::*;
use crate::models::document_with_id::*;
use crate::models::per_datetime::*;
use crate::models::spent_detail::*;
use crate::models::spent_detail_by_es::*;
use crate::models::spent_detail_by_installment::*;
use crate::models::to_python_graph_circle::*;
use crate::models::user_payment_methods::*;

use crate::service_traits::process_service::*;

#[derive(Debug, Getters, Clone, new)]
pub struct ProcessServiceImpl;

impl ProcessServiceImpl {
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
        replacements: &[&str],
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
        consume_price_vec: &[String],
        idx: usize,
    ) -> Result<i64, anyhow::Error> {
        let consume_price: i64 = consume_price_vec
            .get(idx)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_prodt_money()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed.", idx))?
            .parse::<i64>()?;

        Ok(consume_price)
    }

    #[doc = "Function that parses date data and returns DateTime<Local> (internal helper)"]
    /// # Arguments
    /// * `consume_time_name_vec` - Vector with date, time data : ex) ["11/25", "10:02"]
    ///
    /// # Returns
    /// * Result<DateTime<Local>, anyhow::Error>
    fn get_consume_datetime_local(
        &self,
        consume_time_name_vec: &[String],
    ) -> Result<DateTime<Local>, anyhow::Error> {
        /* "11/25" */
        let parsed_date: &String = consume_time_name_vec
            .first()
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_datetime_local()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", 0))?;

        /* "10:02" */
        let parsed_time: &String = consume_time_name_vec
            .get(1)
            .ok_or_else(|| anyhow!("[Index Out Of Range Error][get_consume_datetime_local()] Invalid index '{:?}' of 'consume_time_name_vec' vector was accessed.", 1))?;

        let cur_year: i32 = get_current_kor_naivedate().year();
        let formatted_date_str: String = format!("{}/{}", cur_year, parsed_date);
        let format_date: NaiveDate = NaiveDate::parse_from_str(&formatted_date_str, "%Y/%m/%d")?;
        let format_time: NaiveTime = NaiveTime::parse_from_str(parsed_time, "%H:%M")?;
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

    // #[doc = "Installment filtering function : string -> i64 (internal helper)"]
    // /// # Arguments
    // /// * `payment_type` - Lump sum or installment payment type
    // ///
    // /// # Returns
    // /// * Result<i64, anyhow::Error>
    // fn get_installment_payment_filtering(&self, payment_type: &str) -> Result<i64, anyhow::Error> {
    //     let installment_payment: i64 = match payment_type {
    //         "일시불" => 0,
    //         "03개월" => 3,
    //         "06개월" => 6,
    //         "09개월" => 9,
    //         "12개월" => 12,
    //         _ => 0,
    //     };

    //     Ok(installment_payment)
    // }

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
            .filter(|(_, value)| **value > 0)
            .map(|(key, value)| {
                let prodt_type: String = key.to_string();
                let prodt_cost: i64 = *value;

                let prodt_per: f64 = (prodt_cost as f64 / total_cost) * 100.0;
                let prodt_per_rounded: f64 = (prodt_per * 10.0).round() / 10.0; /* Round to the second decimal place */

                ConsumeResultByType::new(prodt_type, prodt_cost, prodt_per_rounded)
            })
            .collect();

        Ok(consume_result_by_types)
    }

    /// Parses an NH card payment notification message and builds a `SpentDetail`.
    ///
    /// # Arguments
    ///
    /// * `split_args_vec` - Tokenized fields extracted from the notification text
    /// * `user_seq` - Unique identifier of the user
    /// * `room_seq` - Unique identifier of the Telegram room
    /// * `user_payment_methods` - Slice of payment methods registered by the user
    ///
    /// # Returns
    ///
    /// Returns `Ok(SpentDetail)` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or the card alias cannot be matched.
    fn process_nh_card(
        &self,
        split_args_vec: &[String],
        user_seq: i64,
        room_seq: i64,
        user_payment_methods: &[UserPaymentMethods],
    ) -> anyhow::Result<SpentDetail> {
        let split_val: Vec<&str> = vec![",", "원"];

        let card_name: String = split_args_vec
            .first()
            .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_samsung_card] Price field (index 0) not found"))?
            .replace("승인", "");

        let payment_method_id: i64 = user_payment_methods
            .iter()
            .find(|elem| card_name.contains(elem.card_alias().as_str()))
            .map(|elem| *elem.payment_method_id())
            .ok_or_else(|| {
                anyhow!(
                    "[ProcessServiceImpl::process_samsung_card] No matching payment method found for card_name: {}",
                    card_name
                )
            })?;

        // Extract price information
        let price_str: &str = split_args_vec
            .get(2)
            .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_samsung_card] Price field (index 2) not found"))?;
        let consume_price_vec: Vec<String> =
            self.get_string_vector_by_replace(price_str, &split_val)?;
        let spent_money: i64 = self.get_consume_prodt_money(&consume_price_vec, 0)?;

        // Extract time information
        let time_str: &str = split_args_vec
            .get(3)
            .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_samsung_card] Time field (index 3) not found"))?;
        let consume_time_vec: Vec<String> =
            time_str.split(" ").map(|s| s.trim().to_string()).collect();
        let spent_at: DateTime<Local> = self.get_consume_datetime_local(&consume_time_vec)?;

        // Extract product name
        let spent_name: String = split_args_vec
            .get(4)
            .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_samsung_card] Product name field (index 4) not found"))?
            .to_string();

        let spent_detail: SpentDetail = SpentDetail::new(
            spent_name,
            spent_money,
            spent_at,
            1,
            user_seq,
            0,
            0,
            room_seq,
            payment_method_id,
        );

        Ok(spent_detail)
    }

    /// Parses a Samsung card payment notification message and builds a `SpentDetail`.
    ///
    /// # Arguments
    ///
    /// * `split_args_vec` - Tokenized fields extracted from the notification text
    /// * `user_seq` - Unique identifier of the user
    /// * `room_seq` - Unique identifier of the Telegram room
    /// * `user_payment_methods` - Slice of payment methods registered by the user
    ///
    /// # Returns
    ///
    /// Returns `Ok(SpentDetail)` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or the card alias cannot be matched.
    fn process_samsung_card(
        &self,
        split_args_vec: &[String],
        user_seq: i64,
        room_seq: i64,
        user_payment_methods: &[UserPaymentMethods],
    ) -> anyhow::Result<SpentDetail> {
        let split_val: Vec<&str> = vec![",", "원"];

        let card_name: &str = split_args_vec
            .first()
            .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_samsung_card] Price field (index 0) not found"))?;

        let payment_method_id: i64 = user_payment_methods
            .iter()
            .find(|elem| card_name.contains(elem.card_alias().as_str()))
            .map(|elem| *elem.payment_method_id())
            .ok_or_else(|| {
                anyhow!(
                    "[ProcessServiceImpl::process_samsung_card] No matching payment method found for card_name: {}",
                    card_name
                )
            })?;

        // Extract price and payment type
        let price_str = split_args_vec
            .get(1)
            .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_samsung_card] Price field (index 1) not found"))?;
        let consume_price_vec: Vec<String> =
            self.get_string_vector_by_replace(price_str, &split_val)?;
        let spent_money: i64 = self.get_consume_prodt_money(&consume_price_vec, 0)?;

        // Extract time and product name
        let time_str = split_args_vec
            .get(2)
            .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_samsung_card] Time field (index 2) not found"))?;
        let consume_time_vec: Vec<String> = time_str.split(" ").map(|s| s.to_string()).collect();
        let spent_at: DateTime<Local> = self.get_consume_datetime_local(&consume_time_vec)?;

        let spent_name: String = consume_time_vec
            .get(2)
            .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_samsung_card] Product name not found in time field"))?
            .to_string();

        let spent_detail: SpentDetail = SpentDetail::new(
            spent_name,
            spent_money,
            spent_at,
            1,
            user_seq,
            0,
            0,
            room_seq,
            payment_method_id,
        );

        Ok(spent_detail)
    }
}

#[async_trait]
impl ProcessService for ProcessServiceImpl {
    /// Dispatches the card payment notification to the appropriate card-specific parser.
    ///
    /// # Arguments
    ///
    /// * `split_args_vec` - Tokenized fields extracted from the notification text
    /// * `user_seq` - Unique identifier of the user
    /// * `room_seq` - Unique identifier of the Telegram room
    /// * `user_payment_methods` - List of payment methods registered by the user
    ///
    /// # Returns
    ///
    /// Returns `Ok(SpentDetail)` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the card company cannot be identified or parsing fails.
    // 새로운 버전
    fn process_by_consume_filter(
        &self,
        split_args_vec: &[String],
        user_seq: i64,
        room_seq: i64,
        user_payment_methods: Vec<UserPaymentMethods>,
    ) -> anyhow::Result<SpentDetail> {
        let consume_type: &String = split_args_vec
            .first()
            .ok_or_else(|| anyhow!("[Parameter Error][process_by_consume_filter] Invalid format of 'text' variable entered as parameter : {:?}", split_args_vec))?;

        let card_company_nms: HashMap<String, Vec<UserPaymentMethods>> = user_payment_methods
            .into_iter()
            .filter_map(|elem| {
                let nm: String = elem.card_company_nm().clone()?;
                Some((nm, elem))
            })
            .fold(HashMap::new(), |mut acc, (nm, elem)| {
                acc.entry(nm).or_default().push(elem);
                acc
            });

        if card_company_nms.contains_key("nh") && consume_type.contains("nh") {
            let user_payment_methods: &Vec<UserPaymentMethods> = card_company_nms
                .get("nh")
                .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_by_consume_filter_v1] The word ‘NH’ does not exist in the HashMap."))?;

            self.process_nh_card(split_args_vec, user_seq, room_seq, user_payment_methods)
        } else if card_company_nms.contains_key("삼성") && consume_type.contains("삼성") {
            let user_payment_methods: &Vec<UserPaymentMethods> = card_company_nms
                .get("삼성")
                .ok_or_else(|| anyhow!("[ProcessServiceImpl::process_by_consume_filter_v1] The word ‘NH’ does not exist in the HashMap."))?;

            self.process_samsung_card(split_args_vec, user_seq, room_seq, user_payment_methods)
        } else {
            Err(anyhow!("[Error][process_by_consume_filter_v1] Variable 'consume_type' contains an undefined string: {}", consume_type))
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
                    spent_at + chrono::Duration::days(30 * idx);

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
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        nmonth: i32,
    ) -> Result<PerDatetime, anyhow::Error> {
        let n_month_start: DateTime<Utc> = get_add_month_from_naivedate(date_start, nmonth)
            .map_err(|e| anyhow!("[ProcessServiceImpl::get_nmonth_to_current_date] {:?} -> in get_nmonth_to_current_date().n_month_start", e))?;

        let n_month_end: DateTime<Utc> = get_add_month_from_naivedate(date_end, nmonth)
            .map_err(|e| anyhow!("[ProcessServiceImpl::get_nmonth_to_current_date] {:?} -> in get_nmonth_to_current_date().n_month_end", e))?;

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
        spent_details: &AggResultSet<SpentDetailByEs>,
    ) -> Result<Vec<ConsumeResultByType>, anyhow::Error> {
        let spent_inner_details: &Vec<DocumentWithId<SpentDetailByEs>> =
            spent_details.source_list();
        let total_cost: f64 = *spent_details.agg_result();

        let mut cost_map: HashMap<String, i64> =
            spent_inner_details
                .iter()
                .fold(HashMap::new(), |mut acc, spent_detail| {
                    let detail: &SpentDetailByEs = spent_detail.source();
                    let prodt_type: String = detail.consume_keyword_type().to_string();
                    let prodt_money: i64 = detail.spent_money;

                    acc.entry(prodt_type)
                        .and_modify(|value| *value += prodt_money)
                        .or_insert(prodt_money);
                    acc
                });

        cost_map.retain(|_, v| *v > 0);

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
        consume_result_by_types: &[ConsumeResultByType],
        total_cost: f64,
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
    ) -> Result<ToPythonGraphCircle, anyhow::Error> {
        let etc_per: f64 = consume_result_by_types
            .iter()
            .filter(|elem| *elem.consume_prodt_per() <= 3.0)
            .map(|elem| elem.consume_prodt_per())
            .sum();

        let mut entries: Vec<(String, f64)> = consume_result_by_types
            .iter()
            .filter(|elem| *elem.consume_prodt_per() > 3.0)
            .map(|elem| {
                (
                    elem.consume_prodt_type().to_string(),
                    *elem.consume_prodt_per(),
                )
            })
            .collect();

        if etc_per > 0.0 {
            entries.push(("etc".to_string(), etc_per));
        }

        let (prodt_type_vec, prodt_type_cost_per_vec): (Vec<String>, Vec<f64>) =
            entries.into_iter().unzip();

        let to_python_graph_circle: ToPythonGraphCircle = ToPythonGraphCircle::new(
            prodt_type_vec,
            prodt_type_cost_per_vec,
            start_dt.format("%Y-%m-%d").to_string(),
            end_dt.format("%Y-%m-%d").to_string(),
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
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        nday: i32,
    ) -> Result<PerDatetime, anyhow::Error> {
        let n_day_start: DateTime<Utc> = get_add_date_from_naivedate(date_start, nday)?;
        let n_day_end: DateTime<Utc> = get_add_date_from_naivedate(date_end, nday)?;

        let per_day_datetim: PerDatetime =
            PerDatetime::new(date_start, date_end, n_day_start, n_day_end);

        Ok(per_day_datetim)
    }
}
