use crate::common::*;


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


/*

*/
pub fn get_naive_date_from_str(date: &str, format: &str) -> Result<NaiveDate, anyhow::Error> {
    
    let date = match NaiveDate::parse_from_str(date, format) {
        Ok(date) => date,
        Err(e) => return Err(anyhow!(format!("ERROR in 'get_naive_date_from_str()' : {:?}", e)))
    };

    Ok(date)
}





/*
    Functions that make the current date (Korean time) a 'NaiveDateTime' data type
*/
pub fn get_current_kor_naive_datetime() -> NaiveDateTime {

    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    kst_time.naive_local()
}


/*
    Functions that make the current date (Korean time) a 'NaiveDate' data type
*/
pub fn get_current_kor_naivedate() -> NaiveDate {
    
    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    kst_time.date_naive()
}


/*
    Functions that return the first day in 'NaiveDate' format based on the current Korean date
*/
pub fn get_current_kor_naivedate_first_date() -> Result<NaiveDate, anyhow::Error> {

    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    NaiveDate::from_ymd_opt(kst_time.year(), kst_time.month(), 1)
        .ok_or_else(|| anyhow!("Invalid date - get_current_kor_naivedate_first_date() => year: {}, month: {}, day: 1", kst_time.year(), kst_time.month()))
    
}


/*
    Function that obtains the last date of the current month and returns it to 'NaiveDate'
*/
pub fn get_lastday_naivedate(naive_date: NaiveDate) -> Result<NaiveDate, anyhow::Error> {

    let next_month = if naive_date.month() == 12 {
        NaiveDate::from_ymd_opt(naive_date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(naive_date.year(), naive_date.month() + 1, 1)
    }
    .ok_or_else(|| anyhow!("Invalid date when calculating the first day of the next month. - get_lastday_naivedate()"))?;
    
    let last_day_of_month = next_month.pred_opt()
        .ok_or_else(|| anyhow!("Unable to import the previous date for that date. - get_lastday_naivedate()"))?;
    
    Ok(last_day_of_month)
}


/*
    Functions that return NaiveDate data with 'year, month, day' as parameters
*/
pub fn get_naivedate(year: i32, month: u32, date: u32) -> Result<NaiveDate, anyhow::Error> {

    let date = NaiveDate::from_ymd_opt(year, month, date)
        .ok_or_else(|| anyhow!("Invalid date - get_naivedate() => year: {}, month: {}, day: {}", year, month, date))?;
    
    Ok(date)
}


/*
    Function that returns date data a few months before and after a particular date
*/
pub fn get_add_month_from_naivedate(naive_date: NaiveDate, add_month: i32) -> Result<NaiveDate, anyhow::Error> {

    let mut new_year = naive_date.year() + (naive_date.month() as i32 + add_month - 1) / 12;
    let mut new_month = (naive_date.month() as i32 + add_month - 1) % 12 + 1;
    
    // Adjust if the month is out of range
    if new_month <= 0 {
        new_month += 12;
        new_year -= 1;
    }
    
    NaiveDate::from_ymd_opt(new_year, new_month as u32, naive_date.day())
        .ok_or_else(|| anyhow!("Invalid date. - get_add_month_from_naivedate() => new_year: {:?}, new_month: {:?}, day: {:?}", new_year, new_month, naive_date.day()))
}


/*
    Function that checks if the entered string satisfies the reference string format.
*/
pub fn validate_date_format(date_str: &str, format: &str) -> Result<bool, anyhow::Error> {

    let re = Regex::new(format)?;
    Ok(re.is_match(date_str))
}


