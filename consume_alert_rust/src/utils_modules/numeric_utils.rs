#[doc = "Function that determines if the string consists of only numbers"]
pub fn is_numeric(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

#[doc = "Functions that convert strings into numbers"]
pub fn convert_numeric(s: &str) -> i64 {
    s.parse::<i64>().unwrap_or(0)
}
