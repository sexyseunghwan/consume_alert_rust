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
    fn process_by_consume_filter(
        &self,
        split_args_vec: &[String],
        user_seq: i64,
        room_seq: i64,
    ) -> Result<SpentDetailByInstallment, anyhow::Error>;
    async fn process_by_consume_filter_v1(
        &self,
        split_args_vec: &[String],
        user_seq: i64,
        room_seq: i64,
        user_payments: Vec<UserPaymentMethods>
    ) -> anyhow::Result<()>;
    //) -> anyhow::Result<SpentDetail>;
    fn get_spent_detail_installment_process(
        &self,
        spent_detail_by_installment: &SpentDetailByInstallment,
    ) -> Result<Vec<SpentDetail>, anyhow::Error>;
    fn get_nmonth_to_current_date(
        &self,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        nmonth: i32,
    ) -> Result<PerDatetime, anyhow::Error>;
    fn convert_consume_result_by_type_to_python_graph_circle(
        &self,
        consume_result_by_types: &[ConsumeResultByType],
        total_cost: f64,
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
    ) -> Result<ToPythonGraphCircle, anyhow::Error>;
    fn get_consumption_result_by_category(
        &self,
        spent_details: &AggResultSet<SpentDetailByEs>,
    ) -> Result<Vec<ConsumeResultByType>, anyhow::Error>;
    fn get_nday_to_current_date(
        &self,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        nday: i32,
    ) -> Result<PerDatetime, anyhow::Error>;
}
