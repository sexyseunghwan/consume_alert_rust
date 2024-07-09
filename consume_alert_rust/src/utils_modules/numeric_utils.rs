use crate::common::*;

/*
    Function that determines if the string consists of only numbers
*/
pub fn is_numeric(s: &str) -> bool {
    s.parse::<i32>().is_ok()
}


/*
    Functions that convert strings into numbers
*/
pub fn convert_numeric(s: &str) -> i32 {
    match s.parse::<i32>() {
        Ok(num) => num,
        Err(_) => 0,  // Return 0 to default on conversion failure
    }
}