use crate::common::*;

#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct ToPythonGraphCircle {
    prodt_type_vec: Vec<String>,
    prodt_type_cost_per_vec: Vec<f64>,
    start_dt: String,
    end_dt: String,
    total_cost: i64,
}
