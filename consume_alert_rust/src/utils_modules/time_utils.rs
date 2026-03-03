use crate::common::*;

/*
    Function that converts the date data 'DateTime<Utc>' format to the string format "%Y-%m-%d"
*/
pub fn get_str_from_utc_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
}

/*
    Function that converts the date data 'naivedate' format to the string format
*/
pub fn get_str_from_naivedate(naive_date: NaiveDate) -> String {
    naive_date.format("%Y-%m-%d").to_string()
}

/*
    Function that converts the date data 'naivedatetime' format to String format
*/
pub fn get_str_from_naive_datetime(naive_datetime: NaiveDateTime) -> String {
    naive_datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[doc = "Function that returns real-time UTC time to string"]
pub fn get_str_curdatetime() -> String {
    let kor_now: NaiveDateTime = get_current_kor_naive_datetime();
    let formatted_time: String = kor_now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    formatted_time
}

/*
    Function to change 'string' data format to 'NaiveDateTime' format
*/
pub fn get_naive_datetime_from_str(
    date: &str,
    format: &str,
) -> Result<NaiveDateTime, anyhow::Error> {
    NaiveDateTime::parse_from_str(date, format)
        .map_err(|e| anyhow!("[Datetime Parsing Error][get_naive_datetime_from_str()] Failed to parse date string: {:?} : {:?}", date, e))
}

/*
    Function to change 'string' data format to 'NaiveDate' format
*/
pub fn get_naive_date_from_str(date: &str, format: &str) -> Result<NaiveDate, anyhow::Error> {
    NaiveDate::parse_from_str(date, format)
        .map_err(|e| anyhow!("[Datetime Parsing Error][get_naive_date_from_str()] Failed to parse date string: {:?} : {:?}", date, e))
}

/// Parses a date string into a `DateTime<Utc>` (midnight UTC on the parsed date).
pub fn parse_date_as_utc_datetime(
    date: &str,
    format: &str,
) -> Result<DateTime<Utc>, anyhow::Error> {
    NaiveDate::parse_from_str(date, format)
        .map_err(|e| anyhow!("[Datetime Parsing Error][parse_date_as_utc_datetime()] Failed to parse date string: {:?} : {:?}", date, e))
        .map(|d| d.and_time(NaiveTime::MIN).and_utc())
}

/*
    Functions that make the current date (Korean time) a 'NaiveDateTime' data type
*/
pub fn get_current_kor_naive_datetime() -> NaiveDateTime {
    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    kst_time.naive_local()
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

/// Returns the first day of the month that `dt` falls in, as `DateTime<Utc>` at 00:00:00 UTC.
pub fn get_first_day_of_month_start(dt: DateTime<Utc>) -> Result<DateTime<Utc>, anyhow::Error> {
    let naive_date: NaiveDate = dt.date_naive();
    NaiveDate::from_ymd_opt(naive_date.year(), naive_date.month(), 1)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_first_day_of_month_start()] Invalid date => year: {}, month: {}, day: 1",
            naive_date.year(),
            naive_date.month()))
        .map(|d| d.and_time(NaiveTime::MIN).and_utc())
}

/// Returns the last day of the month that `dt` falls in, as `DateTime<Utc>` at 23:59:59 UTC.
pub fn get_last_day_of_month_end(dt: DateTime<Utc>) -> Result<DateTime<Utc>, anyhow::Error> {
    let naive_date: NaiveDate = dt.date_naive();
    let next_month: NaiveDate = if naive_date.month() == 12 {
        NaiveDate::from_ymd_opt(naive_date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(naive_date.year(), naive_date.month() + 1, 1)
    }
    .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_last_day_of_month_end()] Invalid date when calculating the first day of the next month."))?;

    let last_day: NaiveDate = next_month
        .pred_opt()
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_last_day_of_month_end()] Unable to get the previous date."))?;

    let end_time: NaiveTime = NaiveTime::from_hms_opt(23, 59, 59).ok_or_else(|| {
        anyhow!("[Datetime Parsing Error][get_last_day_of_month_end()] Invalid time 23:59:59.")
    })?;

    Ok(last_day.and_time(end_time).and_utc())
}

/// Returns a `DateTime<Utc>` for midnight UTC on the given year/month/day.
pub fn get_naivedate(year: i32, month: u32, date: u32) -> Result<DateTime<Utc>, anyhow::Error> {
    NaiveDate::from_ymd_opt(year, month, date)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_naivedate()] Invalid date => year: {}, month: {}, day: {}", year, month, date))
        .map(|d| d.and_time(NaiveTime::MIN).and_utc())
}

#[doc = "Functions that return NaiveTime data with 'hour, min, sec' as parameters"]
pub fn get_naivetime(hour: u32, min: u32, sec: u32) -> Result<NaiveTime, anyhow::Error> {
    let time = NaiveTime::from_hms_opt(hour, min, sec)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_naivetime()] Invalid date. => hour: {:?}, min: {:?}, sec: {:?}", hour, min, sec))?;

    Ok(time)
}

/*
    Functions that return NaiveDateTime data with 'year, month, day, hour, min, sec' as parameters
*/
pub fn get_naivedatetime(
    year: i32,
    month: u32,
    date: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> Result<NaiveDateTime, anyhow::Error> {
    let naive_date = NaiveDate::from_ymd_opt(year, month, date)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_naivedate()] Invalid date => year: {}, month: {}, day: {}", year, month, date))?;
    let time = get_naivetime(hour, min, sec)?;

    let datetime = NaiveDateTime::new(naive_date, time);

    Ok(datetime)
}

#[doc = "Functions that make NaiveDatetime objects when you add months, days, and hours to the parameters as of the current year"]
pub fn get_this_year_naivedatetime(
    month: u32,
    date: u32,
    hour: u32,
    min: u32,
) -> Result<NaiveDateTime, anyhow::Error> {
    let curr_date: NaiveDateTime = get_current_kor_naive_datetime();

    let date_part = curr_date.date();
    let now_year = date_part.year();

    let datetime = get_naivedatetime(now_year, month, date, hour, min, 0)?;

    Ok(datetime)
}

/// Returns a `DateTime<Utc>` that is `add_month` months after/before `dt`.
pub fn get_add_month_from_naivedate(
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

#[doc = ""]
pub fn convert_str_to_datetime<Tz>(time: &str, tz: Tz) -> anyhow::Result<DateTime<Tz>>
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    let dt_datetime: DateTime<Tz> =
        DateTime::parse_from_rfc3339(time)
        .context("[time_utils.rs -> convert_str_to_datetime] An error occurred during the conversion of time information.")?
        .with_timezone(&tz);

    Ok(dt_datetime)
}
