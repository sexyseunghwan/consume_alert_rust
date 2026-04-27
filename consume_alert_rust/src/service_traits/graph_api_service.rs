use crate::common::*;

use crate::models::to_python_graph_circle::*;
use crate::models::to_python_graph_line::*;

#[async_trait]
pub trait GraphApiService {
    async fn input_api<T: Serialize + Send>(
        &self,
        uri: &str,
        to_python_graph: T,
    ) -> Result<String, anyhow::Error>;

    async fn find_python_matplot_consume_detail_double(
        &self,
        cur_python_graph_info: &ToPythonGraphLine,
        versus_python_graph_info: &ToPythonGraphLine,
    ) -> Result<String, anyhow::Error>;

    async fn find_python_matplot_consume_type(
        &self,
        to_python_graph_circle: &ToPythonGraphCircle,
    ) -> Result<String, anyhow::Error>;
}
