use crate::common::*;

use crate::model::ConsumeInfo::*;
use crate::model::ConsumeTypeInfo::*;
use crate::model::ToPythonGraphCircle::*;
use crate::model::ToPythonGraphLine::*;

#[async_trait]
pub trait GraphApiService {
    async fn post_api<T: Serialize + Send>(
        &self,
        uri: &str,
        to_python_graph: T,
    ) -> Result<String, anyhow::Error>;

    async fn call_python_matplot_consume_type(
        &self,
        consume_type_list: &Vec<ConsumeTypeInfo>,
        start_dt: NaiveDate,
        end_dt: NaiveDate,
        total_cost: f64,
    ) -> Result<String, anyhow::Error>;
    async fn call_python_matplot_consume_detail(
        &self,
        comparison_info: &Vec<ToPythonGraphLine>,
    ) -> Result<String, anyhow::Error>;

    async fn get_consume_type_graph(
        &self,
        total_cost: f64,
        start_dt: NaiveDate,
        end_dt: NaiveDate,
        consume_list: &Vec<ConsumeInfo>,
    ) -> Result<(Vec<ConsumeTypeInfo>, String), anyhow::Error>;
}

#[derive(Debug, Getters)]
pub struct GraphApiServicePub {
    client: Client,
    basic_url: String,
}

impl GraphApiServicePub {
    pub fn new() -> Self {
        let client: Client = reqwest::Client::new();
        let basic_url = String::from("http://localhost:5800");

        Self { client, basic_url }
    }
}

#[async_trait]
impl GraphApiService for GraphApiServicePub {
    #[doc = "docs"]
    async fn post_api<T: Serialize + Send>(
        &self,
        uri: &str,
        to_python_graph: T,
    ) -> Result<String, anyhow::Error> {
        let post_uri = format!("{}{}", self.basic_url, uri);

        let res = self
            .client
            .post(&post_uri)
            .json(&to_python_graph)
            .send()
            .await?;

        if res.status().is_success() {
            let response_body = res.text().await?;
            Ok(response_body)
        } else {
            Err(anyhow!(
                "[Error][cpost_api()] Request for '{}' failed.",
                &post_uri
            ))
        }
    }

    #[doc = "Function that calls python api to draw a pie chart."]
    async fn call_python_matplot_consume_type(
        &self,
        consume_type_list: &Vec<ConsumeTypeInfo>,
        start_dt: NaiveDate,
        end_dt: NaiveDate,
        total_cost: f64,
    ) -> Result<String, anyhow::Error> {
        let mut title_vec: Vec<String> = Vec::new();
        let mut cost_vec: Vec<i32> = Vec::new();

        for consume_elem in consume_type_list {
            let prodt_type = consume_elem.prodt_type();
            let prodt_cost = consume_elem.prodt_cost();

            title_vec.push(prodt_type.to_string());
            cost_vec.push(*prodt_cost)
        }

        let to_python_graph: ToPythonGraphCircle = ToPythonGraphCircle::new(
            title_vec,
            cost_vec,
            start_dt.to_string(),
            end_dt.to_string(),
            total_cost,
        );

        let resp_body = self.post_api("/api/category", to_python_graph).await?;

        Ok(resp_body)
    }

    #[doc = "Function that calls python api to draw a line chart."]
    async fn call_python_matplot_consume_detail(
        &self,
        comparison_info: &Vec<ToPythonGraphLine>,
    ) -> Result<String, anyhow::Error> {
        let resp_body = self
            .post_api("/api/consume_detail", comparison_info)
            .await?;
        Ok(resp_body)
    }

    #[doc = "Function that returns consumption type information (Graphs and detailed consumption details)"]
    async fn get_consume_type_graph(
        &self,
        total_cost: f64,
        start_dt: NaiveDate,
        end_dt: NaiveDate,
        consume_list: &Vec<ConsumeInfo>,
    ) -> Result<(Vec<ConsumeTypeInfo>, String), anyhow::Error> {
        let mut type_scores: HashMap<String, i32> = HashMap::new();

        for consume_info in consume_list {
            let prodt_money = consume_info.prodt_money;
            let prodt_type = consume_info.prodt_type.to_string();

            type_scores
                .entry(prodt_type)
                .and_modify(|e| *e += prodt_money)
                .or_insert(prodt_money);
        }

        let mut consume_type_list: Vec<ConsumeTypeInfo> = Vec::new();

        for (key, value) in &type_scores {
            let prodt_type = key.to_string();
            let prodt_cost = *value;

            if prodt_cost == 0 {
                continue;
            }

            let prodt_per = (prodt_cost as f64 / total_cost) * 100.0;
            let prodt_per_rounded = (prodt_per * 10.0).round() / 10.0; /* Round to the second decimal place */
            
            let consume_type_info = ConsumeTypeInfo::new(prodt_type, prodt_cost, prodt_per_rounded);
            consume_type_list.push(consume_type_info);
        }

        consume_type_list.sort_by(|a, b| b.prodt_cost.cmp(&a.prodt_cost));

        let png_path = self
            .call_python_matplot_consume_type(&consume_type_list, start_dt, end_dt, total_cost)
            .await?;

        Ok((consume_type_list, png_path))
    }
}
