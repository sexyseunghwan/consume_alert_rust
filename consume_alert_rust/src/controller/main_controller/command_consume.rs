use crate::common::*;

use crate::service_traits::elastic_query_service::*;
use crate::service_traits::graph_api_service::*;
use crate::service_traits::mysql_query_service::*;
use crate::service_traits::process_service::*;
use crate::service_traits::producer_service::*;
use crate::service_traits::redis_service::*;
use crate::service_traits::telebot_service::*;

use crate::models::consume_index_prodt_type::*;
use crate::models::spent_detail::*;
use crate::models::spent_detail_to_kafka::*;
use crate::models::spent_detail_with_info::*;
use crate::models::user_payment_methods::*;

use crate::utils_modules::io_utils::*;

use crate::views::spent_detail_view::SpentDetailView;

use super::MainController;

impl<
        G: GraphApiService,
        E: ElasticQueryService,
        M: MysqlQueryService,
        T: TelebotService,
        P: ProcessService,
        KP: ProducerService,
        R: RedisService,
    > MainController<G, E, M, T, P, KP, R>
{
    /// Records a single consumption entry manually from the `c <name>:<amount>` command.
    ///
    /// Parses the name and amount from the input, classifies the spend type via Elasticsearch,
    /// persists the record to MySQL within a transaction, and publishes an insert event to Kafka.
    ///
    /// # Arguments
    ///
    /// * `user_seq` - The authenticated user's sequence number
    /// * `produce_topic` - The Kafka topic to publish the insert event to
    /// * `room_seq` - The Telegram room sequence number used to scope the record
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the record is saved and the confirmation message is sent.
    ///
    /// # Errors
    ///
    /// Returns an error if input parsing, the Elasticsearch query, the MySQL transaction,
    /// the Kafka produce, or the Telegram send fails.
    pub(super) async fn command_consumption(
        &self,
        user_seq: i64,
        produce_topic: &str,
        room_seq: i64,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(":");

        if args.len() != 2 {
            self.tele_bot_service
                .send_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) c snack:15000",
                )
                .await?;
            return Err(anyhow!(
                "[main_controller::command_consumptio] Invalid parameter format: {}",
                self.tele_bot_service.get_input_text()
            ));
        }

        let spent_name: String = args[0].clone();
        let spent_money: i64 = match get_parsed_value_from_vector(&args, 1) {
            Ok(cash) => cash,
            Err(e) => {
                self.tele_bot_service
                    .send_message_confirm(
                        "The second parameter must be numeric.\nEX) c snack:15000",
                    )
                    .await?;
                return Err(anyhow!(
                    "[main_controller::command_consumptio] Non-numeric cash parameter: {:#}",
                    e
                ));
            }
        };

        let spent_type: ConsumingIndexProdtType = self
            .resolve_spend_type(&spent_name)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to insert to MySQL: {:#}",
                    e
                );
            })?;

        let default_payment_method: UserPaymentMethods = match self
            .mysql_query_service
            .get_user_payment_methods(user_seq, true)
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption] Failed to get user payment methods: {:#}", e);
            })?
            .first() {
                Some(default_payment_method) => default_payment_method.clone(),
                None => {
                    self.tele_bot_service
                    .send_message_confirm(
                        "Default payment method does not exist. \n
                        Please register a default payment method.",
                    )
                    .await?;
                    return Err(anyhow!("[main_controller::command_consumption] Default payment method does not exist."))
                }
            };

        let spent_detail: SpentDetail = SpentDetail::new(
            spent_name,
            spent_money,
            Local::now(),
            1,
            user_seq,
            0,
            spent_type.consume_keyword_type_id,
            room_seq,
            default_payment_method.payment_method_id,
        );

        let spent_detail_view: SpentDetailView = spent_detail
            .convert_spent_detail_to_view(&spent_type)
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to build view: {:#}",
                    e
                );
            })?;

        let spent_idx: i64 = self
            .mysql_query_service
            .insert_prodt_detail_with_transaction(&spent_detail)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to insert to MySQL: {:#}",
                    e
                );
            })?;

        let utc_now: DateTime<Utc> = Utc::now();

        let produce_payload: SpentDetailToKafka =
            SpentDetailToKafka::new(spent_idx, String::from("I"), utc_now);

        let partition_key: String = spent_idx.to_string();

        self.producer_service
            .produce_object_to_topic(
                produce_topic,
                &produce_payload,
                Some(partition_key.as_str()),
            )
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to produce Kafka message: {:#}",
                    e
                );
            })?;

        self.tele_bot_service
            .send_message_confirm(&spent_detail_view.to_telegram_string())
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to send Telegram message: {:#}",
                    e
                );
            })?;

        Ok(())
    }

    /// Auto-detects and records a consumption entry from a card-payment notification message.
    ///
    /// Strips bracket-enclosed tokens (e.g. bank prefixes) from the raw input, then delegates
    /// to the process service to extract spend details. Classifies the spend type via
    /// Elasticsearch, persists the record to MySQL within a transaction, and publishes an
    /// insert event to Kafka. Silently returns `Ok(())` if the message is not a payment notice.
    ///
    /// # Arguments
    ///
    /// * `user_seq` - The authenticated user's sequence number
    /// * `produce_topic` - The Kafka topic to publish the insert event to
    /// * `room_seq` - The Telegram room sequence number used to scope the record
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the record is saved and the confirmation message is sent,
    /// or immediately if the input does not resemble a card-payment notification.
    ///
    /// # Errors
    ///
    /// Returns an error if the process filter, Elasticsearch query, MySQL transaction,
    /// Kafka produce, or Telegram send fails.
    pub async fn command_consumption_auto(
        &self,
        user_seq: i64,
        produce_topic: &str,
        room_seq: i64,
    ) -> anyhow::Result<()> {
        let args: String = self.tele_bot_service.get_input_text();

        let bracket_re: Regex = Regex::new(r"\[.*?\]\n?").map_err(|e| {
            anyhow!(
                "[main_controller::command_consumption_auto] Bad regex: {:?}",
                e
            )
        })?;

        let lines: Vec<String> = bracket_re
            .replace_all(&args, "")
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if lines.is_empty() {
            return Ok(());
        }

        let user_payment_methods: Vec<UserPaymentMethods> = self
            .mysql_query_service
            .get_user_payment_methods(user_seq, false)
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to get user payment methods: {:#}", e);
            })?;

        let mut spent_detail: SpentDetail = self
            .process_service
            .process_by_consume_filter(&lines, user_seq, room_seq, user_payment_methods)
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] {:#}", e);
            })?;

        let primary_name: String = spent_detail.spent_name().to_string();

        let spent_type: ConsumingIndexProdtType = self
            .resolve_spend_type(&primary_name)
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to resolve spend type: {:#}", e);
            })?;

        spent_detail.set_consume_keyword_type_id(spent_type.consume_keyword_type_id);

        let spent_detail_view: SpentDetailView = spent_detail
            .convert_spent_detail_to_view(&spent_type)
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption_auto] Failed to build view: {:#}",
                    e
                );
            })?;

        let spent_idx: i64 = self
            .mysql_query_service
            .insert_prodt_detail_with_transaction(&spent_detail)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption_auto] Failed to insert to MySQL: {:#}",
                    e
                );
            })?;

        let utc_now: DateTime<Utc> = Utc::now();

        let produce_payload: SpentDetailToKafka =
            SpentDetailToKafka::new(spent_idx, String::from("I"), utc_now);

        let partition_key: String = spent_idx.to_string();

        self.producer_service
            .produce_object_to_topic(produce_topic, &produce_payload, Some(partition_key.as_str()))
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to produce Kafka message: {:#}", e);
            })?;

        self.tele_bot_service
            .send_message_confirm(&spent_detail_view.to_telegram_string())
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to send Telegram message: {:#}", e);
            })?;
        
        Ok(())
    }

    /// Deletes the most recently recorded consumption entry for the given user and room (`cd`).
    ///
    /// Fetches the latest `SpentDetail` from MySQL, removes it within a transaction, sends a
    /// deletion confirmation to Telegram, and publishes a delete event to Kafka.
    /// Returns early with `Ok(())` if there are no records to delete or the parameter count is wrong.
    ///
    /// # Arguments
    ///
    /// * `produce_topic` - The Kafka topic to publish the delete event to
    /// * `user_seq` - The authenticated user's sequence number
    /// * `room_seq` - The Telegram room sequence number used to scope the query
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the record is deleted and the confirmation message is sent.
    ///
    /// # Errors
    ///
    /// Returns an error if the MySQL query, the delete transaction, the Kafka produce,
    /// or the Telegram send fails.
    pub(super) async fn command_delete_recent_consumption(
        &self,
        produce_topic: &str,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.preprocess_string(" ");

        if args.len() != 1 {
            self.tele_bot_service
                .send_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) cd",
                )
                .await?;
            return Ok(());
        }

        let latest_spent_detail: SpentDetailWithInfo = match self
            .mysql_query_service
            .get_latest_spent_detail(user_seq, room_seq)
            .await?
        {
            Some(latest_spent_detail) => latest_spent_detail,
            None => {
                self.tele_bot_service
                    .send_message_confirm("No expenses to delete.")
                    .await?;
                return Ok(());
            }
        };

        let spent_idx: i64 = latest_spent_detail.spent_idx;

        let spent_detail_view: SpentDetailView = latest_spent_detail.convert_to_view();

        match self
            .mysql_query_service
            .delete_spent_detail_with_transaction(spent_idx)
            .await
        {
            Ok(_) => {
                info!(
                    "[command_delete_recent_consumption] latest spent_idx={} (user_seq={}, room_seq={})",
                    spent_idx, user_seq, room_seq
                );

                self.tele_bot_service
                    .send_message_confirm(&spent_detail_view.to_telegram_string_to_delete())
                    .await?;

                let utc_now: DateTime<Utc> = Utc::now();

                let produce_payload: SpentDetailToKafka =
                    SpentDetailToKafka::new(spent_idx, String::from("D"), utc_now);

                let partition_key: String = spent_idx.to_string();

                self.producer_service
                    .produce_object_to_topic(produce_topic, &produce_payload, Some(partition_key.as_str()))
                    .await
                    .inspect_err(|e| {
                        error!("[main_controller::command_consumption_auto] Failed to produce Kafka message: {:#}", e);
                    })?;
            }
            Err(e) => {
                error!("[command_delete_recent_consumption] Failed delete SPENT_DETAIL information-{}: {:#}", spent_idx, e)
            }
        }

        Ok(())
    }
}
