use crate::common::*;

use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;

#[async_trait]
pub trait GraphApiService {
    /// Sends a POST request with a serializable payload to the given URI and returns the response body.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI path to append to the base graph API URL
    /// * `to_python_graph` - The serializable payload to send as JSON
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` with the response body on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response indicates an error status.
    async fn post_api<T: Serialize + Send>(
        &self,
        uri: &str,
        to_python_graph: T,
    ) -> Result<String, anyhow::Error>;

    /// Calls the Python API to generate a double line chart comparing two consumption periods.
    ///
    /// # Arguments
    ///
    /// * `cur_python_graph_info` - Graph data for the current period
    /// * `versus_python_graph_info` - Graph data for the comparison period
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` with the generated image file path on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request to the Python API fails.
    async fn call_python_matplot_consume_detail_double(
        &self,
        cur_python_graph_info: &ToPythonGraphLine,
        versus_python_graph_info: &ToPythonGraphLine,
    ) -> Result<String, anyhow::Error>;

    /// Calls the Python API to generate a pie chart for consumption categories.
    ///
    /// # Arguments
    ///
    /// * `to_python_graph_circle` - Data transfer object containing category and percentage information
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` with the generated image file path on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request to the Python API fails.
    async fn call_python_matplot_consume_type(
        &self,
        to_python_graph_circle: &ToPythonGraphCircle,
    ) -> Result<String, anyhow::Error>;
}
