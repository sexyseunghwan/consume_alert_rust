use crate::common::*;

/*
    Function that returns the current time in string format  
*/
pub fn get_current_utc_time(time_format: &str) -> String {
    
    let now: DateTime<Utc> = Utc::now();
    
    return now.format(time_format).to_string();
}
