use crate::common::*;

use crate::models::agg_result_set::*;
use crate::models::spent_detail_by_es::*;

#[derive(Debug, Getters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct ToPythonGraphLine {
    line_type: String,
    start_dt: String,
    end_dt: String,
    total_cost: f64,
    consume_accumulate_list: Vec<i64>,
}

impl ToPythonGraphLine {
    pub fn new(
        line_type: &str,
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
        spent_detail: &AggResultSet<SpentDetailByEs>,
    ) -> anyhow::Result<Self> {
        let mut date_consume: HashMap<DateTime<Utc>, i64> = HashMap::new();

        let total_cost: f64 = *spent_detail.agg_result();
        
        for elem in spent_detail.source_list() {
            let elem_date: DateTime<Utc> = elem
                .source
                .spent_at
                .date_naive()
                .and_time(NaiveTime::MIN)
                .and_utc();
            let spent_money: i64 = elem.source.spent_money as i64;

            date_consume
                .entry(elem_date)
                .and_modify(|e| *e += spent_money)
                .or_insert(spent_money);
        }

        let mut sorted_dates: Vec<_> = date_consume.iter().collect(); /* HashMap -> Vector */
        sorted_dates.sort_by(|a, b| a.0.cmp(b.0));

        let sorted_dates_list: Vec<i64> = sorted_dates.into_iter().map(|(_, v)| *v).collect();

        /* List for cumulative total results */
        let mut consume_accumulate_list: Vec<i64> = Vec::new();
        let mut accumulate_cost: i64 = 0;

        for cost in sorted_dates_list {
            accumulate_cost += cost;
            consume_accumulate_list.push(accumulate_cost);
        }

        Ok(ToPythonGraphLine {
            line_type: line_type.to_string(),
            start_dt: start_dt.format("%Y-%m-%d").to_string(),
            end_dt: end_dt.format("%Y-%m-%d").to_string(),
            total_cost,
            consume_accumulate_list,
        })
    }

    // pub fn new(
    //     line_type: &str,
    //     start_dt: DateTime<Utc>,
    //     end_dt: DateTime<Utc>,
    //     consume_detail: &AggResultSet<ConsumeProdtInfo>,
    // ) -> Result<Self, anyhow::Error> {
    //     let mut date_consume: HashMap<NaiveDate, i64> = HashMap::new();

    //     let total_cost: f64 = *consume_detail.agg_result();

    //     for elem in consume_detail.source_list() {
    //         let date_part: &str = elem.source.timestamp.split('T').next().ok_or_else(|| {
    //             anyhow!("[Error][ToPythonGraphLine -> new()] Invalid date. - date_part")
    //         })?;

    //         let elem_date: NaiveDate = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")?;
    //         let prodt_money: i64 = elem.source.prodt_money;

    //         date_consume
    //             .entry(elem_date)
    //             .and_modify(|e| *e += prodt_money)
    //             .or_insert(prodt_money);
    //     }

    //     let mut sorted_dates: Vec<_> = date_consume.iter().collect(); /* HashMap -> Vector */
    //     sorted_dates.sort_by(|a, b| a.0.cmp(b.0));

    //     let sorted_dates_list: Vec<i64> = sorted_dates.into_iter().map(|(_, v)| *v).collect();

    //     /* List for cumulative total results */
    //     let mut consume_accumulate_list: Vec<i64> = Vec::new();
    //     let mut accumulate_cost: i64 = 0;

    //     for cost in sorted_dates_list {
    //         accumulate_cost += cost;
    //         consume_accumulate_list.push(accumulate_cost);
    //     }

    //     Ok(ToPythonGraphLine {
    //         line_type: line_type.to_string(),
    //         start_dt: start_dt.format("%Y-%m-%d").to_string(),
    //         end_dt: end_dt.format("%Y-%m-%d").to_string(),
    //         total_cost,
    //         consume_accumulate_list,
    //     })
    // }

    // #[doc = "Function that adds an amount to the 'Cumulative Consumption Vector'"]
    // pub fn add_to_consume_accumulate_list(&mut self, value: i32) {
    //     self.consume_accumulate_list.push(value);
    // }

    // #[doc = " Function that returns the size of the 'Cumulative Consumption Vector'"]
    // pub fn get_consume_accumulate_list_len(&self) -> usize {
    //     self.consume_accumulate_list.len()
    // }
}
