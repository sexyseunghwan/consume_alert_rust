use crate::common::*;


#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ToPythonGraphCircle {
    title_vec: Vec<String>,
    cost_vec: Vec<i32>,
    start_dt: String,
    end_dt: String,
    total_cost: f64
}
