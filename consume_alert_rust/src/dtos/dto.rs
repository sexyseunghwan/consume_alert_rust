use crate::common::*;

use crate::utils_modules::time_utils::*;


#[derive(Debug, Getters, Serialize, Deserialize, Clone, new)]
#[getset(get = "pub")]
pub struct ConsumeInfo {
    pub timestamp: String,
    pub prodt_name: String,
    pub prodt_money: i32,
    pub prodt_type: String
}

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ProdtDetailInfo {
    pub keyword: String,
    pub bias_value: i32
}

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ProdtTypeInfo {
    pub keyword_type: String,
    pub prodt_detail_list: Vec<ProdtDetailInfo>
}

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ConsumeTypeInfo {
    pub prodt_type: String,
    pub prodt_cost: i32,
    pub prodt_per: f64
}

#[derive(Debug, Getters, Serialize, Deserialize, new)]
#[getset(get = "pub")]
pub struct ToPythonGraphCircle {
    title_vec: Vec<String>,
    cost_vec: Vec<i32>,
    start_dt: String,
    end_dt: String,
    total_cost: f64
}


#[derive(Debug, Getters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct ToPythonGraphLine {
    line_type: String,
    start_dt: String,
    end_dt: String,
    total_cost: f64,
    consume_accumulate_list: Vec<i32>
}

impl ToPythonGraphLine {
    
    pub fn new(line_type: &str, start_dt: &str, end_dt: &str, total_cost: f64, consume_detail: Vec<ConsumeInfo>) -> Result<Self, anyhow::Error> {
        
        let mut consume_accumulate_list: Vec<i32> = Vec::new();
        let mut accumulate_cost = 0;
        
        let mut date: NaiveDate = get_date_from_datestr("1900-01-01T00:00:00Z")?;
        
        for elem in consume_detail {
            let elem_date = get_date_from_datestr(&elem.timestamp)?;
            // Hash map 을 만들어서 구현하는게 나을거 같음.
            // if date == elem_date {
            //     accumulate_cost += elem.prodt_money;
            //     consume_accumulate_list.push(accumulate_cost);
            // } else {

            // }

        }
        
        Ok(
            ToPythonGraphLine {
                line_type: line_type.to_string(),
                start_dt: start_dt.to_string(),
                end_dt: end_dt.to_string(),
                total_cost,
                consume_accumulate_list
            }
        )
    }
}