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
    s.parse::<i32>().unwrap_or(0)
}