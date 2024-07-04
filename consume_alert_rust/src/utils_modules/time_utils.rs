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

*/
pub fn get_one_month_ago_kr_str(date: &str, time_format: &str) -> Result<String, anyhow::Error> {

    println!("{:?}", date);

    let naive_date = NaiveDate::parse_from_str(date, time_format)?;
    let naive_datetime = match naive_date.and_hms_opt(0, 0, 0) {
        Some(naive_datetime) => naive_datetime,
        None => return Err(anyhow!("Invalid date or time provided"))
    };
    
    println!("{:?}", naive_datetime);
    
    let kst_time: DateTime<chrono_tz::Tz> = match Seoul.from_local_datetime(&naive_datetime).single() {
        Some(kst_time) => kst_time,
        None => return Err(anyhow!("Invalid date or time provided"))
    };
    println!("{:?}", kst_time);

    let year = if kst_time.month() == 1 { kst_time.year() - 1 } else { kst_time.year() };
    let month = if kst_time.month() == 1 { 12 } else { kst_time.month() - 1 };
    let day = kst_time.day();
    
    match Seoul.with_ymd_and_hms(year, month, day, 0,0,0) {
        LocalResult::Single(date_time) => Ok(date_time.format(time_format).to_string()),
        _ => Err(anyhow!("Invalid date or time provided")),
    }
}
