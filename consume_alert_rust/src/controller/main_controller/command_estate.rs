use crate::common::*;

use crate::service_traits::cache_service::*;
use crate::service_traits::elastic_query_service::*;
use crate::service_traits::graph_api_service::*;
use crate::service_traits::mysql_query_service::*;
use crate::service_traits::process_service::*;
use crate::service_traits::producer_service::*;
use crate::service_traits::redis_service::*;
use crate::service_traits::telebot_service::*;

use crate::models::earned_detail::*;

use crate::utils_modules::currency_utils::*;
use crate::utils_modules::io_utils::*;

use rust_decimal::Decimal;



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
    /// Saves an earned-detail record entered in Korean won (`ew name:amount`).
    ///
    /// Parses the Telegram command payload, resolves the caller's user sequence,
    /// converts the KRW amount to USD, persists both amounts, and sends a confirmation message.
    ///
    /// # Arguments
    ///
    /// * `telegram_token` - Telegram bot token used to resolve the caller
    /// * `telegram_user_id` - Telegram user id used to resolve the caller
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the earned detail is saved and the confirmation message is sent.
    ///
    /// # Errors
    ///
    /// Returns an error if the command format is invalid, the amount is not numeric,
    /// user resolution fails, currency conversion fails, persistence fails, or Telegram send fails.
    pub(super) async fn command_earend_detail_by_won(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.to_preprocessed_tokens(":");

        if args.len() != 2 {
            self.tele_bot_service
                .input_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) ew salary:5000000",
                )
                .await?;
            return Err(anyhow!(
                "[main_controller::command_earend_detail] Invalid parameter format: {}",
                self.tele_bot_service.get_input_text()
            ));
        }

        let user_seq: i64 = self
            .resolve_user_seq(telegram_token, telegram_user_id)
            .await?;

        let earned_name: String = args[0].clone();
        let earned_money: i64 = match find_parsed_value_from_vector(&args, 1) {
            Ok(cash) => cash,
            Err(e) => {
                self.tele_bot_service
                    .input_message_confirm(
                        "The second parameter must be numeric.\nEX) ew salary:5000000",
                    )
                    .await?;
                return Err(anyhow!(
                    "[main_controller::command_earend_detail] Non-numeric cash parameter: {:#}",
                    e
                ));
            }
        };

        let usd_amount: f64 = krw_to_usd(earned_money)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_earend_detail_by_won] Failed to convert KRW to USD: {:#}",
                    e
                )
            })?;

        let earned_money_dollor: Decimal = Decimal::try_from(usd_amount).map_err(|e| {
            anyhow!(
                "[main_controller::command_earend_detail_by_won] Failed to convert f64 to Decimal: {:#}",
                e
            )
        })?;

        let earned_detail: EarnedDetail = EarnedDetail::new(
            earned_name.clone(),
            earned_money,
            earned_money_dollor,
            Utc::now().into(),
            user_seq,
        );

        self.mysql_query_service
            .input_earned_detail_with_transaction(&earned_detail)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_earend_detail_by_won] Failed to insert to MySQL: {:#}",
                    e
                )
            })?;

        let confirm_msg: String = format!(
            "Earned detail saved!\nName  : {}\nKRW   : {} 원\nUSD   : $ {:.2}",
            earned_name,
            earned_money.to_formatted_string(&Locale::en),
            usd_amount,
        );

        self.tele_bot_service
            .input_message_confirm(&confirm_msg)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_earend_detail_by_won] Failed to send Telegram message: {:#}",
                    e
                )
            })?;

        Ok(())
    }

    /// Saves an earned-detail record entered in US dollars (`ed name:amount`).
    ///
    /// Parses the Telegram command payload, resolves the caller's user sequence,
    /// converts the USD amount to KRW, persists both amounts, and sends a confirmation message.
    ///
    /// # Arguments
    ///
    /// * `telegram_token` - Telegram bot token used to resolve the caller
    /// * `telegram_user_id` - Telegram user id used to resolve the caller
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the earned detail is saved and the confirmation message is sent.
    ///
    /// # Errors
    ///
    /// Returns an error if the command format is invalid, the amount is not numeric,
    /// user resolution fails, currency conversion fails, persistence fails, or Telegram send fails.
    pub(super) async fn command_earend_detail_by_dollor(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.to_preprocessed_tokens(":");

        if args.len() != 2 {
            self.tele_bot_service
                .input_message_confirm(
                    "There is a problem with the parameter you entered. Please check again.\nEX) ed salary:1500.50",
                )
                .await?;
            return Err(anyhow!(
                "[main_controller::command_earend_detail_by_dollor] Invalid parameter format: {}",
                self.tele_bot_service.get_input_text()
            ));
        }

        let user_seq: i64 = self
            .resolve_user_seq(telegram_token, telegram_user_id)
            .await?;

        let earned_name: String = args[0].clone();
        let usd_amount: f64 = match find_parsed_value_from_vector::<f64>(&args, 1) {
            Ok(cash) => cash,
            Err(e) => {
                self.tele_bot_service
                    .input_message_confirm(
                        "The second parameter must be numeric.\nEX) ed salary:1500.50",
                    )
                    .await?;
                return Err(anyhow!(
                    "[main_controller::command_earend_detail_by_dollor] Non-numeric cash parameter: {:#}",
                    e
                ));
            }
        };

        let earned_money: i64 = usd_to_krw(usd_amount)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_earend_detail_by_dollor] Failed to convert USD to KRW: {:#}",
                    e
                )
            })?;

        let earned_money_dollor: Decimal = Decimal::try_from(usd_amount).map_err(|e| {
            anyhow!(
                "[main_controller::command_earend_detail_by_dollor] Failed to convert f64 to Decimal: {:#}",
                e
            )
        })?;

        let earned_detail: EarnedDetail = EarnedDetail::new(
            earned_name.clone(),
            earned_money,
            earned_money_dollor,
            Utc::now().into(),
            user_seq,
        );

        self.mysql_query_service
            .input_earned_detail_with_transaction(&earned_detail)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_earend_detail_by_dollor] Failed to insert to MySQL: {:#}",
                    e
                )
            })?;

        let confirm_msg: String = format!(
            "Earned detail saved!\nName  : {}\nUSD   : $ {:.2}\nKRW   : {} 원",
            earned_name,
            usd_amount,
            earned_money.to_formatted_string(&Locale::en),
        );

        self.tele_bot_service
            .input_message_confirm(&confirm_msg)
            .await
            .inspect_err(|e| {
                error!(
                    "[main_controller::command_earend_detail_by_dollor] Failed to send Telegram message: {:#}",
                    e
                )
            })?;

        Ok(())
    }
}
