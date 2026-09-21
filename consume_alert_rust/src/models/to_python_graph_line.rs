use rust_decimal::prelude::ToPrimitive;

use crate::common::*;

use crate::models::{
    document_with_id::DocumentWithId, spent_detail_by_es::*, spent_detail_by_es_kst::*,
    user_asset_snapshot_summary::*,
};

/// Trait for spent detail types that can be used in graph generation and display
pub trait SpentDetailSource {
    fn spent_money(&self) -> i64;
    fn spent_at(&self) -> DateTime<Utc>;
    fn spent_at_kst(&self) -> DateTime<chrono_tz::Tz>;
    fn spent_name(&self) -> &str;
    fn consume_keyword_type(&self) -> &str;
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
}

/// A single point pluggable into a [`ToPythonGraphLine`]: one value on one KST calendar date.
pub trait GraphLineSource {
    /// The KST calendar date this value belongs to.
    fn line_date_kst(&self) -> NaiveDate;
    /// The value to plot for that date.
    fn line_value(&self) -> i64;
    /// The full KST wall-clock datetime this value belongs to (date + time, no offset).
    fn line_datetime_kst(&self) -> NaiveDateTime;
}

/// Any spending record is a graph line point: its date is `spent_at_kst`, its value is `spent_money`.
impl<T: SpentDetailSource> GraphLineSource for T {
    fn line_date_kst(&self) -> NaiveDate {
        self.spent_at_kst().date_naive()
    }

    fn line_value(&self) -> i64 {
        self.spent_money()
    }

    fn line_datetime_kst(&self) -> NaiveDateTime {
        self.spent_at_kst().naive_local()
    }
}

/// An asset snapshot is a graph line point: its date is `aggregated_at` (KST), its value is
/// `total_asset_amount` (already a running total, unlike spending which must be summed per day).
impl GraphLineSource for UserAssetSnapshotSummary {
    fn line_date_kst(&self) -> NaiveDate {
        self.aggregated_at().with_timezone(&Seoul).date_naive()
    }

    fn line_value(&self) -> i64 {
        self.total_asset_amount()
            .round()
            .to_i64()
            .unwrap_or_default()
    }

    fn line_datetime_kst(&self) -> NaiveDateTime {
        self.aggregated_at().with_timezone(&Seoul).naive_local()
    }
}

/// Delegates to the wrapped source so ES-style `DocumentWithId<T>` lists can be passed directly.
impl<T: GraphLineSource> GraphLineSource for DocumentWithId<T> {
    fn line_date_kst(&self) -> NaiveDate {
        self.source.line_date_kst()
    }

    fn line_value(&self) -> i64 {
        self.source.line_value()
    }

    fn line_datetime_kst(&self) -> NaiveDateTime {
        self.source.line_datetime_kst()
    }
}

/// How same-date values are combined into the plotted list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineAggregation {
    /// Running cumulative sum across sorted dates (e.g. daily spending, total-to-date).
    Cumulative,
    /// Each date's own value, unmodified (e.g. an asset snapshot that is already a running total).
    Raw,
}

#[derive(Debug, Getters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct ToPythonGraphLine {
    line_type: String,
    start_dt: String,
    end_dt: String,
    total_cost: f64,
    accumulate_list: Vec<i64>,
}

impl ToPythonGraphLine {
    /// Builds a `ToPythonGraphLine` from any `GraphLineSource` slice (ES spending records, MySQL
    /// asset snapshots, ...), combining same-date values per `aggregation`.
    ///
    /// # Arguments
    ///
    /// * `line_type` - A label identifying the line series (e.g., `"cur"` or `"versus"`)
    /// * `start_dt` - The start date of the reporting period
    /// * `end_dt` - The end date of the reporting period
    /// * `total_cost` - The headline total shown alongside the line (caller-computed)
    /// * `source_list` - The raw data points to plot
    /// * `aggregation` - Whether same-date values are summed (`Cumulative`) or used as-is (`Raw`)
    ///
    /// # Errors
    ///
    /// Returns an error if construction fails.
    pub fn new<T: GraphLineSource>(
        line_type: &str,
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
        total_cost: f64,
        source_list: &[T],
        aggregation: LineAggregation,
    ) -> anyhow::Result<Self> {
        // Cumulative buckets by KST day (date only); Raw keeps each point's own KST datetime
        // distinct, so same-day snapshots taken at different times aren't collapsed into one.
        let mut date_values: HashMap<NaiveDateTime, i64> = HashMap::new();

        for item in source_list {
            let value: i64 = item.line_value();

            match aggregation {
                LineAggregation::Cumulative => {
                    let kst_day: NaiveDateTime = item.line_date_kst().and_time(NaiveTime::MIN);
                    date_values
                        .entry(kst_day)
                        .and_modify(|e| *e += value)
                        .or_insert(value);
                }
                LineAggregation::Raw => {
                    let kst_datetime: NaiveDateTime = item.line_datetime_kst();
                    date_values.insert(kst_datetime, value);
                }
            }
        }

        let mut sorted_dates: Vec<_> = date_values.into_iter().collect(); /* HashMap -> Vector */
        sorted_dates.sort_by_key(|(datetime, _)| *datetime);

        let accumulate_list: Vec<i64> = match aggregation {
            LineAggregation::Cumulative => {
                let mut accumulate_cost: i64 = 0;
                sorted_dates
                    .into_iter()
                    .map(|(_, value)| {
                        accumulate_cost += value;
                        accumulate_cost
                    })
                    .collect()
            }
            LineAggregation::Raw => sorted_dates.into_iter().map(|(_, value)| value).collect(),
        };

        // Convert start/end to KST for output
        let start_dt_kst: DateTime<chrono_tz::Tz> = start_dt.with_timezone(&Seoul);
        let end_dt_kst: DateTime<chrono_tz::Tz> = end_dt.with_timezone(&Seoul);

        Ok(ToPythonGraphLine {
            line_type: line_type.to_string(),
            start_dt: start_dt_kst.format("%Y-%m-%d").to_string(),
            end_dt: end_dt_kst.format("%Y-%m-%d").to_string(),
            total_cost,
            accumulate_list,
        })
    }
}
