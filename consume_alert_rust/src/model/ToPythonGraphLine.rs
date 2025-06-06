use crate::common::*;

use crate::model::ConsumeInfo::*;

#[derive(Debug, Getters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct ToPythonGraphLine {
    line_type: String,
    start_dt: String,
    end_dt: String,
    total_cost: f64,
    pub consume_accumulate_list: Vec<i32>,
}

impl ToPythonGraphLine {
    pub fn new(
        line_type: &str,
        start_dt: &str,
        end_dt: &str,
        total_cost: f64,
        consume_detail: Vec<ConsumeInfo>,
    ) -> Result<Self, anyhow::Error> {
        let mut date_consume: HashMap<NaiveDate, i32> = HashMap::new();

        for elem in &consume_detail {
            let date_part = elem
                .timestamp
                .split('T')
                .next()
                .ok_or_else(|| anyhow!("Invalid date. - get_add_month_from_naivedate()"))?;

            let elem_date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")?;
            let prodt_money = elem.prodt_money;

            date_consume
                .entry(elem_date)
                .and_modify(|e| *e += prodt_money)
                .or_insert(prodt_money);
        }

        let mut sorted_dates: Vec<_> = date_consume.iter().collect(); /* HashMap -> Vector */
        sorted_dates.sort_by(|a, b| a.0.cmp(b.0));

        let sorted_dates_list: Vec<i32> = sorted_dates.into_iter().map(|(_, v)| *v).collect();
        let mut consume_accumulate_list = Vec::new(); /* cumulative consumption vector */
        let mut accumulate_cost = 0;

        for cost in sorted_dates_list {
            accumulate_cost += cost;
            consume_accumulate_list.push(accumulate_cost);
        }

        Ok(ToPythonGraphLine {
            line_type: line_type.to_string(),
            start_dt: start_dt.to_string(),
            end_dt: end_dt.to_string(),
            total_cost,
            consume_accumulate_list,
        })
    }

    #[doc = "Function that adds an amount to the 'Cumulative Consumption Vector'"]
    pub fn add_to_consume_accumulate_list(&mut self, value: i32) {
        self.consume_accumulate_list.push(value);
    }

    #[doc = " Function that returns the size of the 'Cumulative Consumption Vector'"]
    pub fn get_consume_accumulate_list_len(&self) -> usize {
        self.consume_accumulate_list.len()
    }
}
