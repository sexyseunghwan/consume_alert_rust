use crate::common::*;

use crate::models::agg_result_set::*;
use crate::models::spent_detail_by_es::*;
use crate::models::spent_detail_by_es_kst::*;

/// Trait for spent detail types that can be used in graph generation and display
pub trait SpentDetailSource {
    fn spent_money(&self) -> i64;
    fn spent_at(&self) -> DateTime<Utc>;
    fn spent_at_kst(&self) -> DateTime<chrono_tz::Tz>;
    fn spent_name(&self) -> &str;
    fn consume_keyword_type(&self) -> &str;
    fn consume_keyword_type_id(&self) -> i64;
}

impl SpentDetailSource for SpentDetailByEs {
    fn spent_money(&self) -> i64 {
        self.spent_money
    }

    fn spent_at(&self) -> DateTime<Utc> {
        self.spent_at
    }

    fn spent_at_kst(&self) -> DateTime<chrono_tz::Tz> {
        self.spent_at.with_timezone(&Seoul)
    }

    fn spent_name(&self) -> &str {
        &self.spent_name
    }

    fn consume_keyword_type(&self) -> &str {
        &self.consume_keyword_type
    }

    fn consume_keyword_type_id(&self) -> i64 {
        self.consume_keyword_type_id
    }
}

impl SpentDetailSource for SpentDetailByEsKst {
    fn spent_money(&self) -> i64 {
        self.spent_money
    }

    fn spent_at(&self) -> DateTime<Utc> {
        self.spent_at.with_timezone(&Utc)
    }

    fn spent_at_kst(&self) -> DateTime<chrono_tz::Tz> {
        self.spent_at
    }

    fn spent_name(&self) -> &str {
        &self.spent_name
    }

    fn consume_keyword_type(&self) -> &str {
        &self.consume_keyword_type
    }

    fn consume_keyword_type_id(&self) -> i64 {
        self.consume_keyword_type_id
    }
}

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
    /// Builds a `ToPythonGraphLine` by aggregating daily spending totals into a cumulative list.
    ///
    /// # Arguments
    ///
    /// * `line_type` - A label identifying the line series (e.g., `"cur"` or `"versus"`)
    /// * `start_dt` - The start date of the reporting period
    /// * `end_dt` - The end date of the reporting period
    /// * `spent_detail` - Aggregated result set containing individual spending records
    ///
    /// # Returns
    ///
    /// Returns `Ok(ToPythonGraphLine)` with cumulative daily consumption data on success.
    ///
    /// # Errors
    ///
    /// Returns an error if construction fails.
    pub fn new<T: SpentDetailSource>(
        line_type: &str,
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
        spent_detail: &AggResultSet<T>,
    ) -> anyhow::Result<Self> {
        // Group spending by KST date (date only, no time component)
        let mut date_consume: HashMap<NaiveDate, i64> = HashMap::new();

        let total_cost: f64 = *spent_detail.agg_result();

        for elem in spent_detail.source_list() {
            // Extract KST date (trait method handles UTC→KST conversion if needed)
            let kst_date: NaiveDate = elem.source.spent_at_kst().date_naive();
            let spent_money: i64 = elem.source.spent_money();
            
            date_consume
                .entry(kst_date)
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

        // Convert start/end to KST for output
        let start_dt_kst: DateTime<chrono_tz::Tz> = start_dt.with_timezone(&Seoul);
        let end_dt_kst: DateTime<chrono_tz::Tz> = end_dt.with_timezone(&Seoul);

        Ok(ToPythonGraphLine {
            line_type: line_type.to_string(),
            start_dt: start_dt_kst.format("%Y-%m-%d").to_string(),
            end_dt: end_dt_kst.format("%Y-%m-%d").to_string(),
            total_cost,
            consume_accumulate_list,
        })
    }
}
