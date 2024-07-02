use crate::common::*;

/*
    Function that returns the current time in string format - Timezone: UK 
*/
pub fn get_current_utc_time(time_format: &str) -> String {
    
    let now: DateTime<Utc> = Utc::now();
    
    return now.format(time_format).to_string();
}


/*
    Function that returns the current time in string format - Timezone: Seoul
*/
pub fn get_current_korean_time(time_format: &str) -> String {

    let now: DateTime<Utc> = Utc::now();
    let kst_time: DateTime<chrono_tz::Tz> = now.with_timezone(&Seoul);

    return kst_time.format(time_format).to_string();
}
