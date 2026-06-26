use crate::common::*;

use crate::models::{
    assets::*, stock_pie_data::*, to_python_graph_circle::*, to_python_graph_line::*,
};

#[async_trait]
pub trait GraphApiService {
    // async fn find_python_matplot_consume_detail_double(
    //     &self,
    //     cur_python_graph_info: &ToPythonGraphLine,
    //     versus_python_graph_info: &ToPythonGraphLine,
    // ) -> anyhow::Result<String>;
    async fn find_python_matplot_consume_detail_double(
        &self,
        cur_python_graph_info: &ToPythonGraphLine,
        versus_python_graph_info: &ToPythonGraphLine,
    ) -> anyhow::Result<Vec<u8>>;

    // async fn find_python_matplot_consume_type(
    //     &self,
    //     to_python_graph_circle: &ToPythonGraphCircle,
    // ) -> Result<String, anyhow::Error>;

    async fn find_python_matplot_consume_type(
        &self,
        to_python_graph_circle: &ToPythonGraphCircle,
    ) -> anyhow::Result<Vec<u8>>;

    async fn find_python_matplot_asset_pie(&self, assets: Assets) -> anyhow::Result<Vec<u8>>;

    async fn find_python_matplot_stock_pie(
        &self,
        stock_pie_data: StockPieData,
    ) -> anyhow::Result<Vec<u8>>;
}
