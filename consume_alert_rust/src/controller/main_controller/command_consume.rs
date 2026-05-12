use crate::common::*;

use crate::service_traits::cache_service::*;
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

use crate::AppConfig;

use super::MainController;

impl<
        G: GraphApiService,
        E: ElasticQueryService,
        M: MysqlQueryService,
        T: TelebotService,
        P: ProcessService,
        KP: ProducerService,
        R: RedisService,
        C: CacheService,
    > MainController<G, E, M, T, P, KP, R, C>
{
    /// Records a manual consumption entry from the `c` command (`c item:amount`).
    ///
    /// Validates the command format and amount, resolves the caller and room,
    /// classifies the spending item, loads the user's default payment method,
    /// persists the entry to MySQL, publishes an insert event to Kafka,
    /// and sends a formatted confirmation message to Telegram.
    ///
    /// # Arguments
    ///
    /// * `telegram_token` - Telegram bot token used to resolve the caller
    /// * `telegram_user_id` - Telegram user id used to resolve the caller and room
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the entry is saved and the confirmation is sent.
    ///
    /// # Errors
    ///
    /// Returns an error if the parameter format is invalid, the amount is not numeric,
    /// the caller is unauthorised, no default payment method exists,
    /// or any downstream lookup, persistence, Kafka, or Telegram step fails.
    pub(super) async fn command_consumption(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.to_preprocessed_tokens(":");

        if args.len() != 2 {
            self.tele_bot_service
                .input_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) c snack:15000",
                )
                .await?;
            return Err(anyhow!(
                "[main_controller::command_consumption] Invalid parameter format: {}",
                self.tele_bot_service.get_input_text()
            ));
        }
        
        let user_seq: i64 = self
            .resolve_user_seq(telegram_token, telegram_user_id)
            .await?;

        let room_seq: i64 = self
            .resolve_telegram_room_seq(user_seq, telegram_token, telegram_user_id)
            .await?;
        
        let spent_name: String = args[0].clone();
        let spent_money: i64 = match find_parsed_value_from_vector(&args, 1) {
            Ok(cash) => cash,
            Err(e) => {
                self.tele_bot_service
                    .input_message_confirm(
                        "The second parameter must be numeric.\nEX) c snack:15000",
                    )
                    .await?;
                return Err(anyhow!(
                    "[main_controller::command_consumption] Non-numeric cash parameter: {:#}",
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
            .find_user_payment_methods(user_seq, true)
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption] Failed to get user payment methods: {:#}", e);
            })?
            .first() {
                Some(default_payment_method) => default_payment_method.clone(),
                None => {
                    self.tele_bot_service
                    .input_message_confirm(
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
            Utc::now().with_timezone(&Seoul).fixed_offset(),
            1,
            user_seq,
            0,
            spent_type.consume_keyword_type_id,
            room_seq,
            default_payment_method.payment_method_id,
        );

        let spent_detail_view: SpentDetailView = spent_detail
            .to_spent_detail_view(&spent_type)
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to build view: {:#}",
                    e
                );
            })?;

        let spent_idx: i64 = self
            .mysql_query_service
            .input_prodt_detail_with_transaction(&spent_detail)
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
        let app_config: &AppConfig = AppConfig::get_global();
        let produce_topic: &str = &app_config.produce_topic;

        self.producer_service
            .input_object_to_topic(
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
            .input_message_confirm(&spent_detail_view.to_telegram_string())
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption] Failed to send Telegram message: {:#}",
                    e
                );
            })?;

        Ok(())
    }

    /// Attempts to parse free-form or multi-line text as a consumption entry.
    ///
    /// Removes bracketed metadata fragments such as `[...]`, drops blank lines, resolves the
    /// caller and room, lets `process_service` infer the structured spending data,
    /// classifies the primary spending name, persists the entry to MySQL,
    /// publishes an insert event to Kafka, and sends a confirmation to Telegram.
    /// Returns early with `Ok(())` when no usable lines remain after preprocessing.
    ///
    /// # Arguments
    ///
    /// * `telegram_token` - Telegram bot token used to resolve the caller
    /// * `telegram_user_id` - Telegram user id used to resolve the caller and room
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the parsed entry is saved and the confirmation is sent.
    ///
    /// # Errors
    ///
    /// Returns an error if preprocessing fails, the caller is unauthorised,
    /// payment methods cannot be loaded, the text cannot be converted into a valid entry,
    /// or any downstream persistence, Kafka, or Telegram step fails.
    pub async fn command_consumption_auto(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<()> {
        let args: String = self.tele_bot_service.get_input_text();

        // let bracket_re: Regex = Regex::new(r"\[.*?\]\n?").map_err(|e| {
        //     anyhow!(
        //         "[main_controller::command_consumption_auto] Bad regex: {:?}",
        //         e
        //     )
        // })?;

        let bracket_re: Regex = Regex::new(r"\.*?\\n?").map_err(|e| {
            anyhow!(
                "[main_controller::command_consumption_auto] Bad regex: {:?}",
                e
            )
        })?;
        
        let lines: Vec<String> = bracket_re
            .replace_all(&args, "")
            .split('\n')
            .map(|s| {
                s.replace("[", "")
                    .replace("]", "")
                    .trim()
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        if lines.is_empty() {
            return Ok(());
        }

        let user_seq: i64 = self
            .resolve_user_seq(telegram_token, telegram_user_id)
            .await?;

        let room_seq: i64 = self
            .resolve_telegram_room_seq(user_seq, telegram_token, telegram_user_id)
            .await?;

        let user_payment_methods: Vec<UserPaymentMethods> = self
            .mysql_query_service
            .find_user_payment_methods(user_seq, false)
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to get user payment methods: {:#}", e);
            })?;

        let mut spent_detail: SpentDetail = self
            .process_service
            .modify_by_consume_filter(&lines, user_seq, room_seq, user_payment_methods)
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
            .to_spent_detail_view(&spent_type)
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_consumption_auto] Failed to build view: {:#}",
                    e
                );
            })?;

        let spent_idx: i64 = self
            .mysql_query_service
            .input_prodt_detail_with_transaction(&spent_detail)
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
        
        let app_config: &AppConfig = AppConfig::get_global();
        let produce_topic: &str = app_config.produce_topic();

        self.producer_service
            .input_object_to_topic(produce_topic, &produce_payload, Some(partition_key.as_str()))
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to produce Kafka message: {:#}", e);
            })?;

        self.tele_bot_service
            .input_message_confirm(&spent_detail_view.to_telegram_string())
            .await
            .inspect_err(|e| {
                error!("[main_controller::command_consumption_auto] Failed to send Telegram message: {:#}", e);
            })?;

        Ok(())
    }

    /// Deletes the most recently recorded consumption entry for the caller's room (`cd`).
    ///
    /// Validates that no extra arguments were supplied, resolves the caller and room,
    /// loads the latest spending record from MySQL, removes it inside a transaction,
    /// sends a deletion confirmation to Telegram, and publishes a delete event to Kafka.
    /// Returns early with `Ok(())` if the command format is wrong or there is no record to delete.
    /// If the delete transaction itself fails, the error is logged and the function still returns `Ok(())`.
    ///
    /// # Arguments
    ///
    /// * `telegram_token` - Telegram bot token used to resolve the caller
    /// * `telegram_user_id` - Telegram user id used to resolve the caller and room
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the delete flow completes, or after an early no-op exit.
    ///
    /// # Errors
    ///
    /// Returns an error if authorisation or latest-record lookup fails,
    /// or if any Telegram/Kafka notification step fails.
    /// Delete transaction failures are logged and not propagated.
    pub(super) async fn command_delete_recent_consumption(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.to_preprocessed_tokens(" ");

        if args.len() != 1 {
            self.tele_bot_service
                .input_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) cd",
                )
                .await?;
            return Ok(());
        }
        
        let user_seq: i64 = self
            .resolve_user_seq(telegram_token, telegram_user_id)
            .await?;

        let room_seq: i64 = self
            .resolve_telegram_room_seq(user_seq, telegram_token, telegram_user_id)
            .await?;
        
        let latest_spent_detail: SpentDetailWithInfo = match self
            .mysql_query_service
            .find_latest_spent_detail(user_seq, room_seq)
            .await?
        {
            Some(latest_spent_detail) => latest_spent_detail,
            None => {
                self.tele_bot_service
                    .input_message_confirm("No expenses to delete.")
                    .await?;
                return Ok(());
            }
        };
        
        let spent_idx: i64 = latest_spent_detail.spent_idx;

        let spent_detail_view: SpentDetailView = latest_spent_detail.to_spent_detail_view();

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
                    .input_message_confirm(&spent_detail_view.to_telegram_string_to_delete())
                    .await?;

                let utc_now: DateTime<Utc> = Utc::now();

                let produce_payload: SpentDetailToKafka =
                    SpentDetailToKafka::new(spent_idx, String::from("D"), utc_now);

                let partition_key: String = spent_idx.to_string();

                let app_config: &AppConfig = AppConfig::get_global();
                let produce_topic: &str = app_config.produce_topic();

                self.producer_service
                    .input_object_to_topic(produce_topic, &produce_payload, Some(partition_key.as_str()))
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
