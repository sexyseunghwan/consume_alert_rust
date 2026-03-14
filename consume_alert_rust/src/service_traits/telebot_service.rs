use crate::common::*;

use crate::models::consume_result_by_type::*;
use crate::models::document_with_id::*;
use crate::models::spent_detail_by_es::*;
use crate::models::to_python_graph_line::*;

#[async_trait]
pub trait TelebotService {
    async fn send_message_confirm(&self, msg: &str) -> Result<(), anyhow::Error>;
    async fn send_photo_confirm(&self, image_path_vec: &[String]) -> Result<(), anyhow::Error>;

    async fn send_consumption_message<'life1, 'life2, 'msg, T>(
        &self,
        items: &'life1 [T],
        message_builder: fn(&'life2 T) -> String,
        empty_flag: bool,
        empty_msg: &'msg str,
        msg_title: &'msg str,
    ) -> Result<(), anyhow::Error>
    where
        'life1: 'life2,
        T: Send + Sync;

    async fn send_message_consume_split(
        &self,
        to_python_graph_line: &ToPythonGraphLine,
        spent_detail_list: &[DocumentWithId<SpentDetailByEs>],
    ) -> Result<(), anyhow::Error>;

    async fn send_message_consume_info_by_typelist(
        &self,
        type_consume_info: &[ConsumeResultByType],
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
        total_cost: f64,
    ) -> Result<(), anyhow::Error>;

    fn get_input_text(&self) -> String;

    #[allow(dead_code)]
    async fn send_message_struct_info<T: Serialize + Sync>(
        &self,
        obj_struct: &T,
    ) -> Result<(), anyhow::Error>;

    // async fn send_message_struct_list<T: Serialize + Sync>(
    //     &self,
    //     obj_list: &[T],
    // ) -> Result<(), anyhow::Error>;

    fn get_telegram_token(&self) -> String;
    fn get_telegram_user_id(&self) -> String;
}
