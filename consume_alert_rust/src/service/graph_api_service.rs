use crate::common::*;

use crate::dtos::dto::*;


/*
    Function that calls python api to draw a pie chart
*/
pub async fn call_python_matplot_consume_type(consume_type_list: &Vec<ConsumeTypeInfo>, start_dt: &str, end_dt: &str, total_cost: f64) -> Result<String, anyhow::Error> {

    let client = reqwest::Client::new();

    let mut title_vec: Vec<String> = Vec::new();
    let mut cost_vec: Vec<i32> = Vec::new();

    for consume_elem in consume_type_list {
        let prodt_type = consume_elem.prodt_type();
        let prodt_cost = consume_elem.prodt_cost();

        title_vec.push(prodt_type.to_string());
        cost_vec.push(*prodt_cost)
    }

    let to_python_graph: ToPythonGraphCircle = ToPythonGraphCircle::new(title_vec, cost_vec, start_dt.to_string(), end_dt.to_string(), total_cost);
    
    let res = client.post("http://localhost:5800/api/category")
        .json(&to_python_graph)
        .send()
        .await?;
    
    if res.status().is_success() {
        let response_body = res.text().await?;
        Ok(response_body)
    } else {
        return Err(anyhow!("ERROR in call_python_matplot()"));
    }
    
}