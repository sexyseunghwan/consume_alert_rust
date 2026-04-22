use crate::common::*;

use crate::service_traits::elastic_query_service::*;
use crate::service_traits::graph_api_service::*;
use crate::service_traits::mysql_query_service::*;
use crate::service_traits::process_service::*;
use crate::service_traits::producer_service::*;
use crate::service_traits::redis_service::*;
use crate::service_traits::telebot_service::*;

use crate::models::per_datetime::*;

use crate::configuration::elasitc_index_name::*;
use crate::enums::range_operator::*;
use crate::utils_modules::time_utils::*;

use super::MainController;

impl<
        G: GraphApiService,
        E: ElasticQueryService,
        M: MysqlQueryService,
        T: TelebotService,
        P: ProcessService,
        KP: ProducerService,
        R: RedisService,
    > MainController<G, E, M, T, P, KP, R>
{
    /// Shows the monthly consumption summary for the given room (`cm [YYYY.MM]`).
    ///
    /// Defaults to the current month when no argument is provided.
    /// Accepts an optional `YYYY.MM` argument to query a specific month.
    ///
    /// # Arguments
    ///
    /// * `room_seq` - The Telegram room sequence number used to scope the query
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the summary graphs and messages are sent to Telegram.
    ///
    /// # Errors
    ///
    /// Returns an error if the date argument is invalid, or if any downstream service call fails.
    pub async fn command_consumption_per_mon(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let date_start: DateTime<Utc> = get_current_kor_naivedate_first_date()?;
                let date_end: DateTime<Utc> = get_lastday_naivedate(date_start)?;

                self.process_service
                    .get_nmonth_to_current_date(date_start, date_end, -1)?
            }
            2 if args
                .get(1)
                .is_some_and(|d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) =>
            {
                let parts: Vec<&str> = args[1].split('.').collect();
                let year: i32 = parts
                    .first()
                    .ok_or_else(|| anyhow!("[command_consumption_per_mon] Missing year"))?
                    .parse()?;
                let month: u32 = parts
                    .get(1)
                    .ok_or_else(|| anyhow!("[command_consumption_per_mon] Missing month"))?
                    .parse()?;
                let date_start: DateTime<Utc> = get_naivedate(year, month, 1)?;
                let date_end: DateTime<Utc> = get_lastday_naivedate(date_start)?;
                self.process_service
                    .get_nmonth_to_current_date(date_start, date_end, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "Invalid date format. Please use format YYYY.MM like cm 2023.07",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_mon] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            true,
        )
        .await
    }

    /// Shows the consumption summary for a custom date range (`ctr YYYY.MM.DD-YYYY.MM.DD`).
    ///
    /// Requires a hyphen-separated start and end date in `YYYY.MM.DD` format.
    /// Returns an error if the start date is later than the end date.
    ///
    /// # Arguments
    ///
    /// * `room_seq` - The Telegram room sequence number used to scope the query
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the summary graphs and messages are sent to Telegram.
    ///
    /// # Errors
    ///
    /// Returns an error if the date argument is missing or invalid, the date range is
    /// inverted, or any downstream service call fails.
    pub(super) async fn command_consumption_per_term(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime = match args.len() {
            2 if args.get(1).is_some_and(|d| {
                validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}$")
                    .unwrap_or(false)
            }) =>
            {
                let parts: Vec<&str> = args[1].split('-').collect();
                let start_date: DateTime<Utc> = parse_date_as_utc_datetime(parts[0], "%Y.%m.%d")
                    .inspect_err(|e| {
                        error!(
                            "[command_consumption_per_term] Invalid start date format: {:#}",
                            e
                        )
                    })?;
                let end_date: DateTime<Utc> = parse_date_as_utc_datetime(parts[1], "%Y.%m.%d")
                    .inspect_err(|e| {
                        error!(
                            "[command_consumption_per_term] Invalid end date format: {:#}",
                            e
                        )
                    })?;

                if start_date > end_date {
                    self.tele_bot_service
                        .send_message_confirm(
                            "Invalid date range. The start date must be earlier than or equal to the end date.\nEX) ctr 2023.07.07-2023.08.01",
                        )
                        .await?;
                    return Err(anyhow!(
                        "[command_consumption_per_term] Invalid date range: start_date({}) > end_date({})",
                        start_date.format("%Y.%m.%d"),
                        end_date.format("%Y.%m.%d")
                    ));
                }

                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX) ctr 2023.07.07-2023.08.01",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_term] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            true,
        )
        .await
    }

    /// Shows the daily consumption summary for the given room (`ct [YYYY.MM.DD]`).
    ///
    /// Defaults to today when no argument is provided.
    /// Accepts an optional `YYYY.MM.DD` argument to query a specific date.
    ///
    /// # Arguments
    ///
    /// * `room_seq` - The Telegram room sequence number used to scope the query
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the summary graphs and messages are sent to Telegram.
    ///
    /// # Errors
    ///
    /// Returns an error if the date argument is invalid, or if any downstream service call fails.
    pub(super) async fn command_consumption_per_day(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let today: DateTime<Utc> = get_current_kor_naivedate();
                self.process_service
                    .get_nday_to_current_date(today, today, -1)?
            }
            2 if args.get(1).is_some_and(|d| {
                validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}$").unwrap_or(false)
            }) =>
            {
                let date: DateTime<Utc> = parse_date_as_utc_datetime(&args[1], "%Y.%m.%d")
                    .inspect_err(|e| {
                        error!("[command_consumption_per_day] Invalid date format: {:#}", e)
                    })?;
                self.process_service
                    .get_nday_to_current_date(date, date, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX) ct or ct 2023.11.11",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_day] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            true,
        )
        .await
    }

    /// Shows the consumption summary for the current week (Mon–Sun) (`cw`).
    ///
    /// Calculates the Monday of the current KST week as the start date and the
    /// following Sunday as the end date. Takes no date argument.
    ///
    /// # Arguments
    ///
    /// * `room_seq` - The Telegram room sequence number used to scope the query
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the summary graphs and messages are sent to Telegram.
    ///
    /// # Errors
    ///
    /// Returns an error if any unexpected argument is provided, or if any downstream service call fails.
    pub(super) async fn command_consumption_per_week(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let today: DateTime<Utc> = get_current_kor_naivedate();
                let days_to_monday: i64 = Weekday::Mon.num_days_from_monday() as i64
                    - today.weekday().num_days_from_monday() as i64;
                let monday: DateTime<Utc> = today + chrono::Duration::days(days_to_monday);
                let date_end: DateTime<Utc> = monday + chrono::Duration::days(6);
                self.process_service
                    .get_nday_to_current_date(monday, date_end, -7)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX) cw",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_week] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            true,
        )
        .await
    }

    /// Shows the yearly consumption summary for the given room (`cy [YYYY]`).
    ///
    /// Defaults to the current year when no argument is provided.
    /// Accepts an optional 4-digit `YYYY` argument to query a specific year.
    /// The detail message is suppressed for yearly queries (only graphs are sent).
    ///
    /// # Arguments
    ///
    /// * `room_seq` - The Telegram room sequence number used to scope the query
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the summary graphs and messages are sent to Telegram.
    ///
    /// # Errors
    ///
    /// Returns an error if the year argument is invalid, or if any downstream service call fails.
    pub(super) async fn command_consumption_per_year(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let cur_year = get_current_kor_naivedate().year();
                let start_date: DateTime<Utc> = get_naivedate(cur_year, 1, 1)?;
                let end_date: DateTime<Utc> = get_naivedate(cur_year, 12, 31)?;
                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -12)?
            }
            2 if args
                .get(1)
                .is_some_and(|d| validate_date_format(d, r"^\d{4}$").unwrap_or(false)) =>
            {
                let year: i32 = args[1].parse()?;
                let start_date: DateTime<Utc> = get_naivedate(year, 1, 1)?;
                let end_date: DateTime<Utc> = get_naivedate(year, 12, 31)?;
                self.process_service
                    .get_nmonth_to_current_date(start_date, end_date, -12)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX01) cy\nEX02) cy 2023",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_year] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThanOrEqual,
            room_seq,
            false,
        )
        .await
    }

    /// Shows the consumption summary for the current salary period (`cs [YYYY.MM]`).
    ///
    /// A salary period runs from the 25th of the previous month to the 25th of the current month.
    /// Defaults to the period containing today when no argument is provided.
    /// Accepts an optional `YYYY.MM` argument to query the period ending on the 25th of that month.
    ///
    /// # Arguments
    ///
    /// * `room_seq` - The Telegram room sequence number used to scope the query
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the summary graphs and messages are sent to Telegram.
    ///
    /// # Errors
    ///
    /// Returns an error if the date argument is invalid, or if any downstream service call fails.
    pub async fn command_consumption_per_salary(&self, room_seq: i64) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        let permon_datetime: PerDatetime = match args.len() {
            1 => {
                let today: DateTime<Utc> = get_current_kor_naivedate();
                let (year, month, day) = (today.year(), today.month(), today.day());
                let cur_date_start: DateTime<Utc> = if day < 25 {
                    get_add_month_from_naivedate(get_naivedate(year, month, 25)?, -1)?
                } else {
                    get_naivedate(year, month, 25)?
                };
                let cur_date_end: DateTime<Utc> = if day < 25 {
                    get_naivedate(year, month, 25)?
                } else {
                    get_add_month_from_naivedate(get_naivedate(year, month, 25)?, 1)?
                };
                self.process_service
                    .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
            }
            2 if args
                .get(1)
                .is_some_and(|d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) =>
            {
                let ref_date: DateTime<Utc> =
                    parse_date_as_utc_datetime(&format!("{}.01", args[1]), "%Y.%m.%d")
                        .inspect_err(|e| {
                            error!(
                                "[command_consumption_per_salary] Invalid date format: {:#}",
                                e
                            )
                        })?;
                let cur_date_end: DateTime<Utc> =
                    get_naivedate(ref_date.year(), ref_date.month(), 25)?;
                let cur_date_start: DateTime<Utc> = get_add_month_from_naivedate(cur_date_end, -1)?;
                self.process_service
                    .get_nmonth_to_current_date(cur_date_start, cur_date_end, -1)?
            }
            _ => {
                self.tele_bot_service
                    .send_message_confirm(
                        "There is a problem with the parameter you entered. Please check again.\nEX) cs or cs 2023.11",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_salary] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        self.common_process_python_double(
            &CONSUME_DETAIL,
            permon_datetime,
            RangeOperator::GreaterThanOrEqual,
            RangeOperator::LessThan,
            room_seq,
            true,
        )
        .await
    }
}
