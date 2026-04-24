use crate::common::*;

use crate::models::agg_result_set::*;
use crate::models::consume_result_by_type::*;
use crate::models::per_datetime::*;
use crate::models::spent_detail::*;
use crate::models::spent_detail_by_es::*;
use crate::models::spent_detail_by_installment::*;
use crate::models::to_python_graph_circle::*;
use crate::models::user_payment_methods::*;

#[async_trait]
pub trait ProcessService {
    // fn process_by_consume_filter(
    //     &self,
    //     split_args_vec: &[String],
    //     user_seq: i64,
    //     room_seq: i64,
    // ) -> Result<SpentDetailByInstallment, anyhow::Error>;
    /// Dispatches a card payment notification to the appropriate card-specific parser and returns the parsed spending record.
    ///
    /// # Arguments
    ///
    /// * `split_args_vec` - Tokenized fields from the notification text
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
    /// Returns an error if parsing fails or the card type is unrecognized.
    fn modify_by_consume_filter(
        &self,
        split_args_vec: &[String],
        user_seq: i64,
        room_seq: i64,
        user_payment_methods: Vec<UserPaymentMethods>,
    ) -> anyhow::Result<SpentDetail>;
    //) -> anyhow::Result<SpentDetail>;
    #[allow(dead_code)]
    /// Expands an installment spending record into individual monthly `SpentDetail` entries.
    ///
    /// # Arguments
    ///
    /// * `spent_detail_by_installment` - The spending record with installment count information
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<SpentDetail>)` with one entry per installment month on success.
    ///
    /// # Errors
    ///
    /// Returns an error if date arithmetic fails.
    fn find_spent_detail_installment_process(
        &self,
        spent_detail_by_installment: &SpentDetailByInstallment,
    ) -> Result<Vec<SpentDetail>, anyhow::Error>;
    /// Computes a `PerDatetime` containing the current date range and the same range shifted by `nmonth` months.
    ///
    /// # Arguments
    ///
    /// * `date_start` - Start of the current date range
    /// * `date_end` - End of the current date range
    /// * `nmonth` - Number of months to shift (negative for past, positive for future)
    ///
    /// # Returns
    ///
    /// Returns `Ok(PerDatetime)` with both current and shifted date ranges on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the date arithmetic produces an invalid date.
    fn find_nmonth_to_current_date(
        &self,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        nmonth: i32,
    ) -> Result<PerDatetime, anyhow::Error>;
    /// Converts a list of category consumption results into a `ToPythonGraphCircle` payload for the Python API.
    ///
    /// # Arguments
    ///
    /// * `consume_result_by_types` - Slice of per-category consumption results
    /// * `total_cost` - Total spending amount used as the denominator for percentages
    /// * `start_dt` - Start date of the reporting period
    /// * `end_dt` - End date of the reporting period
    ///
    /// # Returns
    ///
    /// Returns `Ok(ToPythonGraphCircle)` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if construction of the graph payload fails.
    fn to_python_graph_circle_by_consume_type(
        &self,
        consume_result_by_types: &[ConsumeResultByType],
        total_cost: f64,
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
    ) -> Result<ToPythonGraphCircle, anyhow::Error>;
    /// Aggregates spending records by category and returns per-category cost and percentage results.
    ///
    /// # Arguments
    ///
    /// * `spent_details` - Aggregated result set from Elasticsearch containing individual spending documents
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<ConsumeResultByType>)` sorted by cost in descending order on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the calculation fails.
    fn find_consumption_result_by_category(
        &self,
        spent_details: &AggResultSet<SpentDetailByEs>,
    ) -> Result<Vec<ConsumeResultByType>, anyhow::Error>;
    /// Computes a `PerDatetime` containing the current date range and the same range shifted by `nday` days.
    ///
    /// # Arguments
    ///
    /// * `date_start` - Start of the current date range
    /// * `date_end` - End of the current date range
    /// * `nday` - Number of days to shift (negative for past, positive for future)
    ///
    /// # Returns
    ///
    /// Returns `Ok(PerDatetime)` with both current and shifted date ranges on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the date arithmetic produces an invalid date.
    fn find_nday_to_current_date(
        &self,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        nday: i32,
    ) -> Result<PerDatetime, anyhow::Error>;
}
