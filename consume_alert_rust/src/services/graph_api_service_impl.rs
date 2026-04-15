use crate::common::*;

use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;

use crate::service_traits::graph_api_service::*;

static HTTP_CLIENT: once_lazy<Client> = once_lazy::new(initialize_http_clients);

#[doc = "Function to initialize the HTTP client"]
fn initialize_http_clients() -> Client {
    reqwest::Client::new()
}

#[derive(Debug, Getters, Clone)]
pub struct GraphApiServiceImpl {
    graph_api_url: String,
}

impl GraphApiServiceImpl {
    /// Creates a new `GraphApiServiceImpl` by reading the graph API URL from the environment.
    ///
    /// # Returns
    ///
    /// Returns a new `GraphApiServiceImpl` instance.
    pub fn new() -> anyhow::Result<Self> {
        let graph_api_url: String = env::var("GRAPH_API_URL")
            .inspect_err(|e| {
                error!("[GraphApiServiceImpl::new] 'GRAPH_API_URL' must be set: {:#}", e);
            })?;
            
        Ok(Self { graph_api_url })
    }
}

#[async_trait]
impl GraphApiService for GraphApiServiceImpl {
    #[doc = "Function to send post request to 'Python' api"]
    /// # Arguments
    /// * `uri` - uri information
    /// * `to_python_graph` - to_python_graph Object
    ///
    /// # Returns
    /// * Result<String, anyhow::Error>
    async fn post_api<T: Serialize + Send>(
        &self,
        uri: &str,
        to_python_graph: T,
    ) -> Result<String, anyhow::Error> {
        let post_uri: String = format!("{}{}", self.graph_api_url, uri);

        let client: &once_lazy<Client> = &HTTP_CLIENT;

        let res: reqwest::Response = client.post(&post_uri).json(&to_python_graph).send().await?;

        if res.status().is_success() {
            let response_body: String = res.text().await?;
            Ok(response_body)
        } else {
            Err(anyhow!(
                "[Error][post_api()] Request for '{}' failed.",
                &post_uri
            ))
        }
    }

    #[doc = "Function that calls python api to draw a line chart. - double"]
    /// # Arguments
    /// * `cur_python_graph_info` - Objects for representing consumption details in a Python graph
    /// * `versus_python_graph_info` - Objects for representing consumption details in a Python graph (comparative group)
    ///
    /// # Returns
    /// * Result<String, anyhow::Error> -> image file name
    async fn call_python_matplot_consume_detail_double(
        &self,
        cur_python_graph_info: &ToPythonGraphLine,
        versus_python_graph_info: &ToPythonGraphLine,
    ) -> Result<String, anyhow::Error> {
        let python_graph_vec: Vec<ToPythonGraphLine> = vec![
            cur_python_graph_info.clone(),
            versus_python_graph_info.clone(),
        ];

        let resp_body: String = self
            .post_api("/api/consume_detail", python_graph_vec)
            .await?;

        Ok(resp_body)
    }

    /// Calls the Python API to draw a pie chart for consumption categories.
    ///
    /// # Arguments
    ///
    /// * `to_python_graph_circle` - Data transfer object containing category and percentage information for the pie chart
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` containing the generated image file path on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request to the Python API fails.
    async fn call_python_matplot_consume_type(
        &self,
        to_python_graph_circle: &ToPythonGraphCircle,
    ) -> Result<String, anyhow::Error> {
        let resp_body: String = self
            .post_api("/api/category", to_python_graph_circle)
            .await?;
        Ok(resp_body)
    }
}
