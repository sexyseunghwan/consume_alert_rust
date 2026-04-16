use crate::common::*;

/// Parses a date string into a `DateTime<Utc>` (midnight UTC on the parsed date).
pub fn parse_date_as_utc_datetime(
    date: &str,
    format: &str,
) -> Result<DateTime<Utc>, anyhow::Error> {
    NaiveDate::parse_from_str(date, format)
        .map_err(|e| anyhow!("[Datetime Parsing Error][parse_date_as_utc_datetime()] Failed to parse date string: {:?} : {:?}", date, e))
        .map(|d| d.and_time(NaiveTime::MIN).and_utc())
}

/// Returns the current Korean date as `DateTime<Utc>` (midnight UTC on the KST date).
pub fn get_current_kor_naivedate() -> DateTime<Utc> {
    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    kst_time.date_naive().and_time(NaiveTime::MIN).and_utc()
}

/// Returns the first day of the current Korean month as `DateTime<Utc>` (midnight UTC).
pub fn get_current_kor_naivedate_first_date() -> Result<DateTime<Utc>, anyhow::Error> {
    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    NaiveDate::from_ymd_opt(kst_time.year(), kst_time.month(), 1)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_current_kor_naivedate_first_date()] Invalid date => year: {}, month: {}, day: 1",
            kst_time.year(),
            kst_time.month()))
        .map(|d| d.and_time(NaiveTime::MIN).and_utc())
}

/// Returns the last day of the month that `dt` falls in, as `DateTime<Utc>` (midnight UTC).
pub fn get_lastday_naivedate(dt: DateTime<Utc>) -> Result<DateTime<Utc>, anyhow::Error> {
    let naive_date: NaiveDate = dt.date_naive();

    let next_month: NaiveDate = if naive_date.month() == 12 {
        NaiveDate::from_ymd_opt(naive_date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(naive_date.year(), naive_date.month() + 1, 1)
    }
    .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_lastday_naivedate()] Invalid date when calculating the first day of the next month."))?;

    let last_day_of_month: NaiveDate = next_month.pred_opt()
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_lastday_naivedate()] Unable to import the previous date for that date."))?;

    Ok(last_day_of_month.and_time(NaiveTime::MIN).and_utc())
}

/// Returns a `DateTime<Utc>` for midnight UTC on the given year/month/day.
pub fn get_naivedate(year: i32, month: u32, date: u32) -> Result<DateTime<Utc>, anyhow::Error> {
    NaiveDate::from_ymd_opt(year, month, date)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_naivedate()] Invalid date => year: {}, month: {}, day: {}", year, month, date))
        .map(|d| d.and_time(NaiveTime::MIN).and_utc())
}

/// Returns a `DateTime<Utc>` that is `add_month` months after/before `dt`.
///
/// Uses [`chrono::Months`] and `checked_add_months`/`checked_sub_months` to avoid
/// the truncating-division bug present in the old implementation when `add_month` is negative.
/// When the resulting month has fewer days than the source day, the day is clamped to the
/// last valid day of that month (chrono's built-in behaviour).
///
/// # Arguments
///
/// * `dt` - The base UTC datetime
/// * `add_month` - Number of months to add (negative value subtracts months)
///
/// # Returns
///
/// Returns `Ok(DateTime<Utc>)` on success, or an error if the resulting date is out of range.
pub fn get_add_month_from_naivedate(
    dt: DateTime<Utc>,
    add_month: i32,
) -> Result<DateTime<Utc>, anyhow::Error> {
    let naive_date: NaiveDate = dt.date_naive();
    let result_date: NaiveDate = if add_month >= 0 {
        naive_date
            .checked_add_months(Months::new(add_month as u32))
            .ok_or_else(|| anyhow!("[time_utils::get_add_month_from_naivedate] Date overflow when adding {} months to {:?}", add_month, naive_date))?
    } else {
        naive_date
            .checked_sub_months(Months::new((-add_month) as u32))
            .ok_or_else(|| anyhow!("[time_utils::get_add_month_from_naivedate] Date underflow when subtracting {} months from {:?}", -add_month, naive_date))?
    };
    
    Ok(result_date.and_time(NaiveTime::MIN).and_utc())
}

#[allow(dead_code)]
/// Returns a `DateTime<Utc>` that is `add_month` months after/before `dt`.
pub fn get_add_month_from_naivedate_old(
    dt: DateTime<Utc>,
    add_month: i32,
) -> Result<DateTime<Utc>, anyhow::Error> {
    let naive_date = dt.date_naive();
    let mut new_year: i32 = naive_date.year() + (naive_date.month() as i32 + add_month - 1) / 12;
    let mut new_month: i32 = (naive_date.month() as i32 + add_month - 1) % 12 + 1;

    /* Adjust if the month is out of range */
    if new_month <= 0 {
        new_month += 12;
        new_year -= 1;
    }

    /*
        Handling Date Data Exception.
        ex) 2024-11-31 -> Not exists
    */
    let mut input_day: u32 = naive_date.day();

    let new_date: NaiveDate = NaiveDate::from_ymd_opt(new_year, new_month as u32, 1)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_add_month_from_naivedate()] Invalid date => year: {}, month: {}, day: 1", new_year, new_month))?;
    let next_month = if new_date.month() == 12 {
        NaiveDate::from_ymd_opt(new_date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(new_date.year(), new_date.month() + 1, 1)
    };

    let last_day: u32 = next_month
        .ok_or_else(|| anyhow!("[Error][get_add_month_from_naivedate()] Problem with variable 'last_day'"))?
        .pred_opt()
        .ok_or_else(|| anyhow!("[Error][get_add_month_from_naivedate()] Problem while converting variable 'last_day'"))?
        .day();

    if input_day > last_day {
        input_day = last_day;
    }

    NaiveDate::from_ymd_opt(new_year, new_month as u32, input_day)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_add_month_from_naivedate()] Invalid date. => new_year: {:?}, new_month: {:?}, day: {:?}",
            new_year,
            new_month,
            input_day))
        .map(|d| d.and_time(NaiveTime::MIN).and_utc())
}

/// Returns a `DateTime<Utc>` that is `add_day` days after/before `dt`.
pub fn get_add_date_from_naivedate(
    dt: DateTime<Utc>,
    add_day: i32,
) -> Result<DateTime<Utc>, anyhow::Error> {
    let naive_date = dt.date_naive();
    let duration = chrono::Duration::days(add_day.into());
    let result_date = naive_date.checked_add_signed(duration).ok_or_else(|| {
        anyhow!("[Error][get_add_date_from_naivedate()] Invalid date calculation")
    })?;

    Ok(result_date.and_time(NaiveTime::MIN).and_utc())
}

#[doc = "Function that checks if the entered string satisfies the reference string format."]
pub fn validate_date_format(date_str: &str, format: &str) -> Result<bool, anyhow::Error> {
    let re = Regex::new(format)?;
    Ok(re.is_match(date_str))
}

/// Formats a `DateTime<Utc>` value as a KST (Korea Standard Time) string using the given format pattern.
///
/// # Arguments
///
/// * `dt` - The UTC datetime to convert and format
/// * `format` - The `strftime`-style format string (e.g., `"%Y-%m-%dT%H:%M"`)
///
/// # Returns
///
/// Returns the formatted KST datetime string.
pub fn format_kst_datetime(dt: DateTime<Utc>, format: &str) -> String {
    let kst: DateTime<chrono_tz::Tz> = dt.with_timezone(&Seoul);
    kst.format(format).to_string()
}
