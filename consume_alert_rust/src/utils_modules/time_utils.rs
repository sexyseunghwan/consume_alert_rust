use crate::common::*;

/*
    Function that returns the current time in string format - Timezone: UK 
*/
pub fn get_current_utc_time_str(time_format: &str) -> String {
    
    let now: DateTime<Utc> = Utc::now();
    
    return now.format(time_format).to_string();
}

/*
    Function that returns the current time in string format - Timezone: Seoul
*/
pub fn get_current_korean_time_str(time_format: &str) -> String {

    let now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = now.with_timezone(&Seoul);

    return kst_time.format(time_format).to_string();
}


/*

*/
// pub fn get_one_month_ago_kr_str(date: DateTime<Utc>, time_format: &str) -> String {

//     let kst_time: DateTime<chrono_tz::Tz> = date.with_timezone(&Seoul);

//     let year = if kst_time.month() == 1 { kst_time.year() - 1 } else { kst_time.year() };
//     let month = if kst_time.month() == 1 { 12 } else { kst_time.month() - 1 };
//     let day = kst_time.day();
    

    
//     //let res = Utc.ymd(year, month, day).and_hms(date.hour(), date.minute(), date.second());


// }
