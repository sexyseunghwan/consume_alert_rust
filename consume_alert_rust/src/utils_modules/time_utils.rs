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
pub fn get_date_from_fulldate(date_str: &str) -> Result<String, anyhow::Error> {
    
    let datetime: NaiveDateTime = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%3fZ")?;
    let date: NaiveDate = datetime.date();
    let formatted_date = date.format("%Y-%m-%d").to_string();

    Ok(formatted_date)
}

/*
    Function that calculates the last date of a particular month
*/
pub fn get_last_date_str(date_str: &str, format: &str) -> Result<String, anyhow::Error> {

    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;

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
    
    let naive_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
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




// =========================================================================================================


/*

*/
pub fn get_str_from_naivedate(naive_date: NaiveDate) -> String {
    naive_date.format("%Y-%m-%d").to_string()
}


/*

*/
pub fn get_current_kor_naivedate() -> NaiveDate {
    
    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    kst_time.date_naive()
}


/*

*/
pub fn get_current_kor_naivedate_first_date() -> Result<NaiveDate, anyhow::Error> {

    let utc_now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = utc_now.with_timezone(&Seoul);

    NaiveDate::from_ymd_opt(kst_time.year(), kst_time.month(), 1)
        .ok_or_else(|| anyhow!("Invalid date for year: {}, month: {}, day: 1", kst_time.year(), kst_time.month()))
    
}

/*
    
*/
pub fn get_lastday_naivedate(naive_date: NaiveDate) -> Result<NaiveDate, anyhow::Error> {

    let next_month = if naive_date.month() == 12 {
        NaiveDate::from_ymd_opt(naive_date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(naive_date.year(), naive_date.month() + 1, 1)
    }
    .ok_or_else(|| anyhow!("Invalid date when calculating the first day of the next month."))?;
    
    let last_day_of_month = next_month.pred_opt()
        .ok_or_else(|| anyhow!("Unable to import the previous date for that date."))?;
    
    Ok(last_day_of_month)
}


/*

*/
//pub fn get_add_month_from_naivedate(naive_date: NaiveDate, add_month: i32) -> Result<NaiveDate, anyhow::Error> {

pub fn get_add_month_from_naivedate(naive_date: NaiveDate, add_month: i32) -> Result<(), anyhow::Error> {

    let mut new_year = naive_date.year() + (naive_date.month() as i32 + add_month - 1) / 12;
    let mut new_month = (naive_date.month() as i32 + add_month - 1) % 12 + 1;
    
    println!("new_year: {:?}", new_year);
    println!("new_month: {:?}", new_month);
        
    Ok(())
    // let next_month = if naive_date.month() == 12 {
    //     NaiveDate::from_ymd_opt(naive_date.year() + 1, 1, 1)
    // } else {
    //     NaiveDate::from_ymd_opt(naive_date.year(), naive_date.month() + 1, 1)
    // }
    // .ok_or_else(|| anyhow!("Invalid date when calculating the first day of the next month."))?;

}


// pub fn get_add_month_from_naivedate(naive_date: NaiveDate, add_month: i32) -> Result<NaiveDate, anyhow::Error> {
//     // 계산된 새로운 연도와 월을 구합니다.
//     let mut new_year = naive_date.year() + (naive_date.month() as i32 + add_month - 1) / 12;
//     let mut new_month = (naive_date.month() as i32 + add_month - 1) % 12 + 1;

//     // 월이 범위를 벗어나면 조정
//     if new_month <= 0 {
//         new_month += 12;
//         new_year -= 1;
//     }

//     // 새로운 날짜를 구하되, 월의 마지막 날을 고려
//     match NaiveDate::from_ymd_opt(new_year, new_month as u32, naive_date.day()) {
//         Some(date) => Ok(date),
//         None => {
//             // 입력된 일이 목표 월의 일수를 초과하는 경우 해당 월의 마지막 날을 사용
//             let last_day_of_new_month = NaiveDate::from_ymd(new_year, new_month as u32, 1).succ_opt().unwrap().pred();
//             Ok(last_day_of_new_month)
//         }
//     }
// }

