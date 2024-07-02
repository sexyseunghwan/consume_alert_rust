use crate::common::*;

/*
    Function that determines if the string consists of only numbers
*/
pub fn is_numeric(s: &str) -> bool {
    s.parse::<i32>().is_ok()
}