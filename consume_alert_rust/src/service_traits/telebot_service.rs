use crate::common::*;

use crate::models::consume_result_by_type::*;
use crate::models::document_with_id::*;
use crate::models::spent_detail_by_es::*;
use crate::models::to_python_graph_line::*;

#[async_trait]
pub trait TelebotService {
    /// Sends a text message to the Telegram chat, retrying on failure.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message text to send
    ///
    /// # Errors
    ///
    /// Returns an error if all retry attempts fail.
    async fn send_message_confirm(&self, msg: &str) -> Result<(), anyhow::Error>;

    /// Sends one or more photos to the Telegram chat, retrying each on failure.
    ///
    /// # Arguments
    ///
    /// * `image_path_vec` - Slice of file paths pointing to images to send
    ///
    /// # Errors
    ///
    /// Returns an error if any photo fails to send after all retries.
    async fn send_photo_confirm(&self, image_path_vec: &[String]) -> Result<(), anyhow::Error>;

    /// Sends a list of items as formatted Telegram messages, paginating every 10 items.
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to format and send
    /// * `message_builder` - Function that formats a single item into a string
    /// * `empty_flag` - If `true`, sends `empty_msg` instead of item details
    /// * `empty_msg` - Message to send when the list is empty
    /// * `msg_title` - Title prepended to the first message batch
    ///
    /// # Errors
    ///
    /// Returns an error if sending any message fails.
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

    /// Sends a summary of consumption details for a date range along with graph metadata.
    ///
    /// # Arguments
    ///
    /// * `to_python_graph_line` - Graph metadata including date range and total cost
    /// * `spent_detail_list` - List of individual spending documents to display
    ///
    /// # Errors
    ///
    /// Returns an error if sending any message fails.
    async fn send_message_consume_split(
        &self,
        to_python_graph_line: &ToPythonGraphLine,
        spent_detail_list: &[DocumentWithId<SpentDetailByEs>],
    ) -> Result<(), anyhow::Error>;

    /// Sends a per-category consumption summary message for the specified date range.
    ///
    /// # Arguments
    ///
    /// * `type_consume_info` - Slice of per-category consumption results
    /// * `start_dt` - Start date of the reporting period
    /// * `end_dt` - End date of the reporting period
    /// * `total_cost` - Total spending amount for the period
    ///
    /// # Errors
    ///
    /// Returns an error if sending any message fails.
    async fn send_message_consume_info_by_typelist(
        &self,
        type_consume_info: &[ConsumeResultByType],
        start_dt: DateTime<Utc>,
        end_dt: DateTime<Utc>,
        total_cost: f64,
    ) -> Result<(), anyhow::Error>;

    /// Returns the raw input text received from the Telegram message.
    ///
    /// # Returns
    ///
    /// Returns the input text as a `String`.
    fn get_input_text(&self) -> String;

    /// Serializes a struct and sends its field values as a Telegram message.
    ///
    /// # Arguments
    ///
    /// * `obj_struct` - A serializable struct to format and send
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or sending fails.
    #[allow(dead_code)]
    async fn send_message_struct_info<T: Serialize + Sync>(
        &self,
        obj_struct: &T,
    ) -> Result<(), anyhow::Error>;

    // async fn send_message_struct_list<T: Serialize + Sync>(
    //     &self,
    //     obj_list: &[T],
    // ) -> Result<(), anyhow::Error>;

    /// Returns the Telegram bot token string.
    ///
    /// # Returns
    ///
    /// Returns the bot token as a `String`.
    fn get_telegram_token(&self) -> String;

    /// Returns the Telegram user ID string.
    ///
    /// # Returns
    ///
    /// Returns the user ID as a `String`.
    fn get_telegram_user_id(&self) -> String;
}
