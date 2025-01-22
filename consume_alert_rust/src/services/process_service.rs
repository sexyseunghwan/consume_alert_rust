use crate::common::*;

use crate::utils_modules::time_utils::*;

use crate::models::consume_prodt_info::*;
use crate::models::per_datetime::*;

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
    fn process_by_consume_filter(
        &self,
        split_args_vec: &Vec<String>,
    ) -> Result<ConsumeProdtInfo, anyhow::Error>;
    fn get_nmonth_to_current_date(
        &self,
        date_start: NaiveDate,
        date_end: NaiveDate,
        nmonth: i32,
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
    /// * `consume_price_vec`  -
    /// * `idx`-
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
    ) -> Result<ConsumeProdtInfo, anyhow::Error> {
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

            Ok(res_struct)
        } else if consume_type.contains("삼성") {
            let consume_price_vec = self.get_string_vector_by_replace(split_args_vec
                .get(1)
                .ok_or_else(|| anyhow!("[Index Out Of Range Error][process_by_consume_type()] Invalid index '{:?}' of 'consume_price_vec' vector was accessed. : {:?}", 1, split_args_vec))?,
                &split_val
            )?;

            let consume_price: i64 = self.get_consume_prodt_money(&consume_price_vec, 0)?;

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

            Ok(res_struct)
        } else {
            return Err(anyhow!("[Error][process_by_consume_type()] Variable 'consume_type' contains an undefined string."));
        }
    }

    #[doc = "Function that returns the time allotted as a parameter and the time before/after `N` months"]
    /// # Arguments
    /// * `date_start`  
    /// * `date_end`    
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
}
