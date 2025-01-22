use crate::common::*;

use crate::models::to_python_graph_line::*;

static HTTP_CLIENT: once_lazy<Client> = once_lazy::new(|| initialize_http_clients());

#[doc = "Function to initialize the HTTP client"]
fn initialize_http_clients() -> Client {
    reqwest::Client::new()
}

#[async_trait]
pub trait GraphApiService {
    async fn post_api<T: Serialize + Send>(
        &self,
        uri: &str,
        to_python_graph: T,
    ) -> Result<String, anyhow::Error>;
    async fn call_python_matplot_consume_detail(
        &self,
        comparison_info: &Vec<ToPythonGraphLine>,
    ) -> Result<String, anyhow::Error>;
}

#[derive(Debug, Getters, Clone)]
pub struct GraphApiServicePub {
    graph_api_url: String,
}

impl GraphApiServicePub {
    pub fn new() -> Self {
        let graph_api_url: String = env::var("GRAPH_API_URL").expect(
            "[ENV file read Error][GraphApiServicePub -> new()] 'GRAPH_API_URL' must be set",
        );

        Self { graph_api_url }
    }
}

#[async_trait]
impl GraphApiService for GraphApiServicePub {
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

    #[doc = ""]
    /// # Arguments
    /// * `comparison_info` - uri information
    ///
    /// # Returns
    /// * Result<String, anyhow::Error>
    async fn call_python_matplot_consume_detail(
        &self,
        comparison_info: &Vec<ToPythonGraphLine>,
    ) -> Result<String, anyhow::Error> {
        Ok(String::from("test"))
    }
}
