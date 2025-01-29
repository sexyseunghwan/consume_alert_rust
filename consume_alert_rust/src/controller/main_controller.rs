use crate::common::*;

use crate::services::elastic_query_service::*;
use crate::services::graph_api_service::*;
use crate::services::mysql_query_service::*;
use crate::services::process_service::*;
use crate::services::telebot_service::*;

use crate::utils_modules::io_utils::*;
use crate::utils_modules::time_utils::*;

use crate::repository::es_repository::*;

use crate::configuration::elasitc_index_name::*;

use crate::models::agg_result_set::*;
use crate::models::consume_prodt_info::*;
use crate::models::consume_result_by_type::*;
use crate::models::document_with_id::*;
use crate::models::per_datetime::*;
use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;

pub struct MainController<
    G: GraphApiService,
    E: ElasticQueryService,
    M: MysqlQueryService,
    T: TelebotService,
    P: ProcessService,
> {
    graph_api_service: Arc<G>,
    elastic_query_service: Arc<E>,
    mysql_query_service: Arc<M>,
    tele_bot_service: T,
    process_service: Arc<P>,
}

impl<
        G: GraphApiService,
        E: ElasticQueryService,
        M: MysqlQueryService,
        T: TelebotService,
        P: ProcessService,
    > MainController<G, E, M, T, P>
{
    pub fn new(
        graph_api_service: Arc<G>,
        elastic_query_service: Arc<E>,
        mysql_query_service: Arc<M>,
        tele_bot_service: T,
        process_service: Arc<P>,
    ) -> Self {
        Self {
            graph_api_service,
            elastic_query_service,
            mysql_query_service,
            tele_bot_service,
            process_service,
        }
    }

    #[doc = "Function that processes the request when the request is received through telegram bot"]
    pub async fn main_call_function(&self) -> Result<(), anyhow::Error> {
        let input_text: String = self.tele_bot_service.get_input_text();

        match input_text.split_whitespace().next().unwrap_or("") {
            "c" => self.command_consumption().await?,
            "cd" => self.command_delete_recent_cunsumption().await?,
            "cm" => self.command_consumption_per_mon().await?,
            "ctr" => self.command_consumption_per_term().await?,
            "ct" => self.command_consumption_per_day().await?,
            "cs" => self.command_consumption_per_salary().await?,
            "cw" => self.command_consumption_per_week().await?,
            // "mc" => self.command_record_fasting_time().await?,
            // "mt" => self.command_check_fasting_time().await?,
            // "md" => self.command_delete_fasting_time().await?,
            "cy" => self.command_consumption_per_year().await?,
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

    #[doc = "Common Processing Controller Function -> Responsible for Python API calls."]
    /// # Arguments
    /// * `index_name` - Index name
    /// * `permon_datetime` - Structures with date data to compare with date
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn common_process_python_double(
        &self,
        index_name: &str,
        permon_datetime: PerDatetime,
    ) -> Result<(), anyhow::Error> {
        let consume_detail_info: AggResultSet<ConsumeProdtInfo> = self
            .elastic_query_service
            .get_info_orderby_aggs_range(
                index_name,
                "@timestamp",
                permon_datetime.date_start,
                permon_datetime.date_end,
                "@timestamp",
                true,
                "prodt_money",
            )
            .await?;

        let versus_consume_detail_info: AggResultSet<ConsumeProdtInfo> = self
            .elastic_query_service
            .get_info_orderby_aggs_range(
                index_name,
                "@timestamp",
                permon_datetime.n_date_start,
                permon_datetime.n_date_end,
                "@timestamp",
                true,
                "prodt_money",
            )
            .await?;

        let cur_python_graph: ToPythonGraphLine = ToPythonGraphLine::new(
            "cur",
            permon_datetime.date_start,
            permon_datetime.date_end,
            &consume_detail_info,
        )?;

        let versus_python_graph: ToPythonGraphLine = ToPythonGraphLine::new(
            "versus",
            permon_datetime.n_date_start,
            permon_datetime.n_date_end,
            &versus_consume_detail_info,
        )?;

        /* The consumption details are sent through the Telegram bot. */
        self.tele_bot_service
            .send_message_consume_split(&cur_python_graph, &consume_detail_info.source_list())
            .await?;

        /* Using Python API */
        let mut img_files: Vec<String> = Vec::new();

        /* ======== Graph of consumption details - image path ======== */
        let cnosume_detail_img_file_path: String = self
            .graph_api_service
            .call_python_matplot_consume_detail_double(&cur_python_graph, &versus_python_graph)
            .await?;

        img_files.push(cnosume_detail_img_file_path);

        /* ======== Graph of consumption type - image path ======== */
        let consume_result_by_type: Vec<ConsumeResultByType> = self
            .process_service
            .get_consumption_result_by_category(&consume_detail_info)?;

        let to_python_circle_graph: ToPythonGraphCircle = self
            .process_service
            .convert_consume_result_by_type_to_python_graph_circle(
                &consume_result_by_type,
                *consume_detail_info.agg_result(),
                permon_datetime.date_start,
                permon_datetime.date_end,
            )?;

        let python_circle_graph_path: String = self
            .graph_api_service
            .call_python_matplot_consume_type(&to_python_circle_graph)
            .await?;

        img_files.push(python_circle_graph_path);

        /* Send consumption details graph photo */
        self.tele_bot_service.send_photo_confirm(&img_files).await?;

        /* The consumption details are summarized and shown by category. */
        self.tele_bot_service
            .send_message_consume_info_by_typelist(
                &consume_result_by_type,
                permon_datetime.date_start,
                permon_datetime.date_end,
                *consume_detail_info.agg_result(),
            )
            .await?;

        /* Delete Image file */
        delete_file(img_files)?;

        Ok(())
    }

    //fn common_process_python_single(&self, )

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

        let con_index_prod: ConsumeProdtInfo = ConsumeProdtInfo::new(
            cur_time.clone(),
            cur_time.clone(),
            consume_name.to_string(),
            consume_cash_i64,
            prodt_type,
        );

        let document: Value = convert_json_from_struct(&con_index_prod)?;

        let es_client: EsRepositoryPub = get_elastic_conn()?;
        es_client.post_query(&document, &CONSUME_DETAIL).await?;

        self.tele_bot_service
            .send_message_struct_info(&con_index_prod)
            .await?;

        Ok(())
    }

    #[doc = "command handler: Writes the expenditure details to the index in ElasticSearch."]
    pub async fn command_consumption_auto(&self) -> Result<(), anyhow::Error> {
        let args: String = self.tele_bot_service.get_input_text();

        let es_conn: EsRepositoryPub = get_elastic_conn()?;

        let re: Regex = Regex::new(r"\[.*?\]\n?")?;
        let replace_string: String = re.replace_all(&args, "").to_string(); /* Remove the '[~]' string. */

        let split_args_vec: Vec<String> = replace_string
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(); /* It convert the string into an array */

        let mut filter_consume_info: ConsumeProdtInfo = self
            .process_service
            .process_by_consume_filter(&split_args_vec)?;

        /* It determines the type of consumption. */
        let consume_type: String = self
            .elastic_query_service
            .get_consume_type_judgement(filter_consume_info.prodt_name())
            .await?;

        filter_consume_info.set_prodt_type(consume_type);

        /* Index that object to Elasticsearch. */
        es_conn
            .post_query_struct(&filter_consume_info, &CONSUME_DETAIL)
            .await?;

        self.tele_bot_service
            .send_message_struct_info(&filter_consume_info)
            .await?;

        Ok(())
    }

    #[doc = "command handler: Function to erase the most recent consumption history data -> cd"]
    pub async fn command_delete_recent_cunsumption(&self) -> Result<(), anyhow::Error> {
        let split_args_vec: Vec<String> = self.preprocess_string(" ");

        match split_args_vec.len() {
            1 => {
                let recent_consume_info: Vec<DocumentWithId<ConsumeProdtInfo>> = self
                    .elastic_query_service
                    .get_info_orderby_cnt(&CONSUME_DETAIL, "cur_timestamp", 1, false)
                    .await?;

                let top_consume_data: &DocumentWithId<ConsumeProdtInfo> = recent_consume_info
                    .get(0)
                    .ok_or_else(|| anyhow!("[Error][command_delete_recent_cunsumption()] Data 'top_consume_data' does not exist."))?;

                /* Delete Index */
                self.elastic_query_service
                    .delete_es_doc(&CONSUME_DETAIL, top_consume_data)
                    .await?;

                let consume_info: &ConsumeProdtInfo = top_consume_data.source();

                /* To confirm the deleted document. */
                self.tele_bot_service
                    .send_message_struct_info(consume_info)
                    .await?;
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) cd")
                    .await?;
            }
        }

        Ok(())
    }

    #[doc = "command handler: Checks how much you have consumed during a month -> cm"]
    pub async fn command_consumption_per_mon(&self) -> Result<(), anyhow::Error> {
        let split_args_vec: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {
                let date_start: NaiveDate = get_current_kor_naivedate_first_date()?;
                let date_end: NaiveDate = get_lastday_naivedate(date_start)?;

                self.process_service
                    .get_nmonth_to_current_date(date_start, date_end, -1)?
            }
            2 if split_args_vec.get(1).map_or(false, |d| {
                validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)
            }) =>
            {
                let dates: Vec<&str> = split_args_vec[1].split('.').collect::<Vec<&str>>();

                let year: i32 = dates.get(0)
                    .ok_or_else(|| anyhow!("[Error][command_consumption_per_mon()] 'year' variable has not been initialized."))?
                    .parse()?;

                let month: u32 = dates.get(1)
                    .ok_or_else(|| anyhow!("[Error][command_consumption_per_mon()] 'month' variable has not been initialized."))?
                    .parse()?;

                let date_start: NaiveDate = get_naivedate(year, month, 1)?;
                let date_end: NaiveDate = get_lastday_naivedate(date_start)?;

                self.process_service
                    .get_nmonth_to_current_date(date_start, date_end, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "Invalid date format. Please use format YYYY.MM like cm 2023.07",
                    )
                    .await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_mon()] Invalid format of 'text' variable entered as parameter. : {:?}", self.tele_bot_service.get_input_text()));
            }
        };

        self.common_process_python_double(&CONSUME_DETAIL, permon_datetime)
            .await?;

        Ok(())
    }

    #[doc = "command handler: Checks how much you have consumed during a specific periods -> ctr"]
    async fn command_consumption_per_term(&self) -> Result<(), anyhow::Error> {
        let split_args_vec: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            2 if split_args_vec.get(1).map_or(false, |d| {
                validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}$")
                    .unwrap_or(false)
            }) =>
            {
                let dates = split_args_vec[1].split('-').collect::<Vec<&str>>();

                let start_date = NaiveDate::parse_from_str(dates[0], "%Y.%m.%d")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_term()] This does not fit the date format : {:?}", e))?;

                let end_date = NaiveDate::parse_from_str(dates[1], "%Y.%m.%d")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_term()] This does not fit the date format : {:?}", e))?;

                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) ctr 2023.07.07-2023.08.01")
                    .await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_term()] Invalid format of 'text' variable entered as parameter. : {:?}", self.tele_bot_service.get_input_text()));
            }
        };

        self.common_process_python_double(&CONSUME_DETAIL, permon_datetime)
            .await?;

        Ok(())
    }

    #[doc = "command handler: Checks how much you have consumed during a day -> ct"]
    async fn command_consumption_per_day(&self) -> Result<(), anyhow::Error> {
        let split_args_vec: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {
                let start_dt = get_current_kor_naivedate();
                let end_dt = get_current_kor_naivedate();

                self.process_service
                    .get_nday_to_current_date(start_dt, end_dt, -1)?
            }
            2 if split_args_vec.get(1).map_or(false, |d| {
                validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}$").unwrap_or(false)
            }) =>
            {
                let cur_date = NaiveDate::parse_from_str(&split_args_vec[1], "%Y.%m.%d")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_day()] This does not fit the date format : {:?}", e))?;

                self.process_service
                    .get_nday_to_current_date(cur_date, cur_date, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) ct or ct 2023.11.11").await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_day()] Invalid format of 'text' variable entered as parameter. : {:?}", self.tele_bot_service.get_input_text()));
            }
        };

        self.common_process_python_double(&CONSUME_DETAIL, permon_datetime)
            .await?;

        Ok(())
    }

    #[doc = "command handler: Checks how much you have consumed during a week -> cw"]
    async fn command_consumption_per_week(&self) -> Result<(), anyhow::Error> {
        let split_args_vec: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {
                let now: NaiveDateTime = get_current_kor_naive_datetime();
                let today: NaiveDate = now.date();
                let weekday: Weekday = today.weekday();

                let days_until_monday = Weekday::Mon.num_days_from_monday() as i64
                    - weekday.num_days_from_monday() as i64;
                let monday: NaiveDate = today + chrono::Duration::days(days_until_monday);

                let date_start: NaiveDate = monday + chrono::Duration::days(0);
                let date_end: NaiveDate = monday + chrono::Duration::days(6);

                self.process_service
                    .get_nday_to_current_date(date_start, date_end, -7)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) cw").await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_week()] Invalid format of 'text' variable entered as parameter. : {:?}", self.tele_bot_service.get_input_text()));
            }
        };

        self.common_process_python_double(&CONSUME_DETAIL, permon_datetime)
            .await?;

        Ok(())
    }

    #[doc = "command handler: Checks how much you have consumed during one year -> cy"]
    async fn command_consumption_per_year(&self) -> Result<(), anyhow::Error> {
        let split_args_vec: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {
                let cur_year: i32 = get_current_kor_naivedate().year();
                let start_date: NaiveDate = get_naivedate(cur_year, 1, 1)?;
                let end_date: NaiveDate = get_naivedate(cur_year, 12, 31)?;

                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -12)?
            }
            2 if split_args_vec.get(1).map_or(false, |d| {
                validate_date_format(d, r"^\d{4}$").unwrap_or(false)
            }) =>
            {
                let year: i32 = split_args_vec[1].parse::<i32>()?;
                let start_date: NaiveDate = get_naivedate(year, 1, 1)?;
                let end_date: NaiveDate = get_naivedate(year, 12, 31)?;

                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -12)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX01) cy\nEX02) cy 2023").await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_year()] Invalid format of 'text' variable entered as parameter. : {:?}", self.tele_bot_service.get_input_text()));
            }
        };

        self.common_process_python_double(&CONSUME_DETAIL, permon_datetime)
            .await?;

        Ok(())
    }

    #[doc = "command handler: Check the consumption details from the date of payment to the next payment. -> cs"]
    pub async fn command_consumption_per_salary(&self) -> Result<(), anyhow::Error> {
        let split_args_vec: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match split_args_vec.len() {
            1 => {
                let cur_day: NaiveDate = get_current_kor_naivedate();
                let cur_year: i32 = cur_day.year();
                let cur_month: u32 = cur_day.month();
                let cur_date: u32 = cur_day.day();

                let cur_date_start: NaiveDate = if cur_date < 25 {
                    let date: NaiveDate = get_naivedate(cur_year, cur_month, 25)?;
                    get_add_month_from_naivedate(date, -1)?
                } else {
                    get_naivedate(cur_year, cur_month, 25)?
                };

                let cur_date_end: NaiveDate = if cur_date < 25 {
                    get_naivedate(cur_year, cur_month, 25)?
                } else {
                    let date: NaiveDate = get_naivedate(cur_year, cur_month, 25)?;
                    get_add_month_from_naivedate(date, 1)?
                };

                self.process_service
                    .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
            }
            2 if split_args_vec.get(1).map_or(false, |d| {
                validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)
            }) =>
            {
                let curdate_str: String = format!("{}.01", &split_args_vec[1]);
                let cur_date: NaiveDate = NaiveDate::parse_from_str(&curdate_str, "%Y.%m.%d")
                        .map_err(|e| anyhow!("[Error][command_consumption_per_salary()] This does not fit the date format : {:?}", e))?;

                let cur_date_end: NaiveDate = get_naivedate(cur_date.year(), cur_date.month(), 25)?;
                let cur_date_start: NaiveDate = get_add_month_from_naivedate(cur_date_end, -1)?;

                self.process_service
                    .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm("There is a problem with the parameter you entered. Please check again. \nEX) cs or cs 2023.11").await?;

                return Err(anyhow!("[Parameter Error][command_consumption_per_day()] Invalid format of 'text' variable entered as parameter. : {:?}", self.tele_bot_service.get_input_text()));
            }
        };

        self.common_process_python_double(&CONSUME_DETAIL, permon_datetime)
            .await?;

        Ok(())
    }
}
