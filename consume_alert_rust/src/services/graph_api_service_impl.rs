use crate::common::*;

use crate::models::{
    assets::*, stock_pie_data::*, to_python_graph_circle::*, to_python_graph_line::*,
};

use crate::service_traits::graph_api_service::*;

static HTTP_CLIENT: once_lazy<Client> = once_lazy::new(initialize_http_clients);

#[doc = "Function to initialize the HTTP client"]
fn initialize_http_clients() -> Client {
    reqwest::Client::new()
}

#[derive(Debug, Getters, Clone)]
pub struct GraphApiServiceImpl {
    graph_api_url: reqwest::Url,
}

impl GraphApiServiceImpl {
    /// Creates a new `GraphApiServiceImpl` by reading the graph API URL from the environment.
    ///
    /// # Returns
    ///
    /// Returns a new `GraphApiServiceImpl` instance.
    pub fn new() -> anyhow::Result<Self> {
        let raw_url: String = env::var("GRAPH_API_URL").inspect_err(|e| {
            error!(
                "[GraphApiServiceImpl::new] 'GRAPH_API_URL' must be set: {:#}",
                e
            );
        })?;

        let graph_api_url = reqwest::Url::parse(&raw_url).map_err(|e| {
            anyhow!(
                "[GraphApiServiceImpl::new] Invalid GRAPH_API_URL '{}': {}",
                raw_url,
                e
            )
        })?;

        Ok(Self { graph_api_url })
    }

    async fn call_python_graph_api_bytes<T: Serialize + Send>(
        &self,
        uri: &str,
        body: T,
    ) -> anyhow::Result<Vec<u8>> {
        let post_uri: Url = self.graph_api_url.join(uri).map_err(|e| {
            anyhow!(
                "[GraphApiServiceImpl::call_python_graph_api_bytes] Invalid URI '{}': {}",
                uri,
                e
            )
        })?;

        let client: &once_lazy<Client> = &HTTP_CLIENT;

        let res: reqwest::Response = client.post(post_uri.clone()).json(&body).send().await?;

        if res.status().is_success() {
            Ok(res.bytes().await?.to_vec())
        } else {
            let status: reqwest::StatusCode = res.status();
            let error_body: String = res.text().await.unwrap_or_default();
            Err(anyhow!(
                "[GraphApiServiceImpl::call_python_graph_api_bytes] Request for '{}' failed. Status: {}, Body: {}",
                post_uri,
                status,
                error_body
            ))
        }
    }
}

#[async_trait]
impl GraphApiService for GraphApiServiceImpl {
    async fn find_python_matplot_consume_detail_double(
        &self,
        cur_python_graph_info: &ToPythonGraphLine,
        versus_python_graph_info: &ToPythonGraphLine,
    ) -> anyhow::Result<Vec<u8>> {
        let python_graph_vec: Vec<ToPythonGraphLine> = vec![
            cur_python_graph_info.clone(),
            versus_python_graph_info.clone(),
        ];

        self.call_python_graph_api_bytes("/api/consume_detail", python_graph_vec)
            .await
    }

    async fn find_python_matplot_consume_type(
        &self,
        to_python_graph_circle: &ToPythonGraphCircle,
    ) -> anyhow::Result<Vec<u8>> {
        self.call_python_graph_api_bytes("/api/category", to_python_graph_circle)
            .await
    }

    async fn find_python_matplot_asset_pie(&self, assets: Assets) -> anyhow::Result<Vec<u8>> {
        self.call_python_graph_api_bytes("/api/asset_pie_image_app", assets)
            .await
    }

    async fn find_python_matplot_stock_pie(
        &self,
        stock_pie_data: StockPieData,
    ) -> anyhow::Result<Vec<u8>> {
        self.call_python_graph_api_bytes("/api/stock_pie_image", stock_pie_data)
            .await
    }
}
