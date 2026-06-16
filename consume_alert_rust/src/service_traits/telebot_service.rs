use crate::common::*;

use crate::models::{
    consume_result_by_type::*, document_with_id::*, file_info::*, to_python_graph_line::*,
};

#[async_trait]
pub trait TelebotService {
    async fn input_message_confirm(&self, msg: &str) -> Result<(), anyhow::Error>;
    async fn input_photo_confirm(&self, image_vecs: Vec<FileInfo>) -> anyhow::Result<()>;
    async fn input_photo_from_bytes(
        &self,
        bytes: Vec<u8>,
        filename: &str,
    ) -> Result<(), anyhow::Error>;

    async fn input_consumption_message<'life1, 'life2, 'msg, T>(
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

    async fn input_message_consume_split<T: SpentDetailSource + Send + Sync>(
        &self,
        to_python_graph_line: &ToPythonGraphLine,
        spent_detail_list: &[DocumentWithId<T>],
    ) -> Result<(), anyhow::Error>;

    async fn input_message_consume_info_by_typelist(
        &self,
        type_consume_info: &[ConsumeResultByType],
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
        total_cost: f64,
    ) -> Result<(), anyhow::Error>;

    fn get_input_text(&self) -> String;

    fn get_telegram_token(&self) -> String;

    fn get_telegram_user_id(&self) -> String;
}
