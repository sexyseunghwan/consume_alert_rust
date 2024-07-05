use crate::common::*;

/*
    Function that returns the current time in string format - Timezone: UK 
*/
pub fn get_current_utc_time_str(time_format: &str) -> String {
    
    let now: DateTime<Utc> = Utc::now();
    
    now.format(time_format).to_string()
}

/*
    Function that returns the current time in string format - Timezone: Seoul
*/
pub fn get_current_korean_time_str(time_format: &str) -> String {

    let now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = now.with_timezone(&Seoul);

    kst_time.format(time_format).to_string()
}


/*
    Function that checks if the entered string satisfies the reference string format.
*/
pub fn validate_date_format(date_str: &str, format: &str) -> Result<bool, anyhow::Error> {

    let re = Regex::new(format)?;
    Ok(re.is_match(date_str))
}


/*
    
*/
pub fn get_last_date_str(date_str: &str, format: &str) -> Result<String, anyhow::Error> {

    let date = NaiveDate::parse_from_str(date_str, "%Y.%m.%d")?;

    let next_month = if date.month() == 12 { 1 } else { date.month() + 1 };
    let next_month_year = if date.month() == 12 { date.year() + 1 } else { date.year() };

    let temp_date = match NaiveDate::from_ymd_opt(next_month_year, next_month, 1) {
        Some(temp_date) => temp_date,
        None => return Err(anyhow!("Date conversion failed"))
    };

    let last_day = match temp_date.pred_opt() {
        Some(last_day) => last_day,
        None => return Err(anyhow!("Date conversion failed"))
    };

    Ok(last_day.format(format).to_string())
}


/*
    Function that calculates date information for the first day of a month ago
*/
pub fn get_one_month_ago_kr_str(date: &str, time_format: &str) -> Result<String, anyhow::Error> {
    
    let naive_date = NaiveDate::parse_from_str(date, "%Y.%m.%d")?;
    let naive_datetime = match naive_date.and_hms_opt(0, 0, 0) {
        Some(naive_datetime) => naive_datetime,
        None => return Err(anyhow!("Invalid date or time provided"))
    };

    let kst_time: DateTime<chrono_tz::Tz> = match Seoul.from_local_datetime(&naive_datetime).single() {
        Some(kst_time) => kst_time,
        None => return Err(anyhow!("Invalid date or time provided"))
    };

    let year = if kst_time.month() == 1 { kst_time.year() - 1 } else { kst_time.year() };
    let month = if kst_time.month() == 1 { 12 } else { kst_time.month() - 1 };
    let day = kst_time.day();

    match Seoul.with_ymd_and_hms(year, month, day, 0,0,0) {
        LocalResult::Single(date_time) => Ok(date_time.format(time_format).to_string()),
        _ => Err(anyhow!("Invalid date or time provided")),
    }
}
