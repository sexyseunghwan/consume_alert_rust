use crate::common::*;

use crate::dtos::MainControllerServicesDto;
use crate::service_traits::{
    cache_service::*, elastic_query_service::*, graph_api_service::*, mysql_query_service::*,
    process_service::*, producer_service::*, redis_service::*, telebot_service::*,
};

mod command_asset;
mod command_consume;
mod command_python_call;
mod command_query;
mod command_resolver;

pub struct MainController<
    G: GraphApiService,
    E: ElasticQueryService,
    M: MysqlQueryService,
    T: TelebotService,
    P: ProcessService,
    KP: ProducerService,
    R: RedisService,
    C: CacheService,
> {
    pub(super) graph_api_service: Arc<G>,
    pub(super) elastic_query_service: Arc<E>,
    pub(super) mysql_query_service: Arc<M>,
    pub(super) tele_bot_service: T,
    pub(super) process_service: Arc<P>,
    pub(super) producer_service: Arc<KP>,
    #[allow(dead_code)]
    pub(super) redis_service: Arc<R>,
    pub(super) cache_service: Arc<C>,
}

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
    pub fn new(services: MainControllerServicesDto<G, E, M, T, P, KP, R, C>) -> Self {
        Self {
            graph_api_service: services.graph_api_service,
            elastic_query_service: services.elastic_query_service,
            mysql_query_service: services.mysql_query_service,
            tele_bot_service: services.tele_bot_service,
            process_service: services.process_service,
            producer_service: services.producer_service,
            redis_service: services.redis_service,
            cache_service: services.cache_service,
        }
    }

    /// Dispatches the current Telegram input to the matching command handler.
    ///
    /// Reads the bot token, Telegram user id, and raw input text from `tele_bot_service`,
    /// then routes to the appropriate handler based on the first whitespace-delimited token.
    /// Authentication and room resolution are performed inside each command handler.
    /// Unrecognised input falls through to the auto-consumption parser.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the selected handler completes successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the Telegram context or executing the selected handler fails.
    pub async fn main_call_function(&self) -> anyhow::Result<()> {
        let telegram_token: String = self.tele_bot_service.get_telegram_token();
        let telegram_user_id: String = self.tele_bot_service.get_telegram_user_id();
        let input_text: String = self.tele_bot_service.get_input_text();

        match input_text.split_whitespace().next().unwrap_or("") {
            "c" => {
                self.command_consumption(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cd" => {
                self.command_delete_recent_consumption(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cm" => {
                self.command_consumption_per_mon(&telegram_token, &telegram_user_id)
                    .await?
            }
            "ctr" => {
                self.command_consumption_per_term(&telegram_token, &telegram_user_id)
                    .await?
            }
            "ct" => {
                self.command_consumption_per_day(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cs" => {
                self.command_consumption_per_salary(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cw" => {
                self.command_consumption_per_week(&telegram_token, &telegram_user_id)
                    .await?
            }
            "cy" => {
                self.command_consumption_per_year(&telegram_token, &telegram_user_id)
                    .await?
            }
            "gs" => {
                self.command_consumption_per_salary_group(&telegram_token, &telegram_user_id)
                    .await?
            }
            "gm" => {
                self.command_consumption_per_mon_group(&telegram_token, &telegram_user_id)
                    .await?
            }
            "gt" => {
                self.command_consumption_per_day_group(&telegram_token, &telegram_user_id)
                    .await?
            }
            "gw" => {
                self.command_consumption_per_week_group(&telegram_token, &telegram_user_id)
                    .await?
            }
            "gy" => {
                self.command_consumption_per_year_group(&telegram_token, &telegram_user_id)
                    .await?
            }
            "ew" => {
                self.command_earend_detail_by_won(&telegram_token, &telegram_user_id)
                    .await?
            }
            "ed" => {
                self.command_earend_detail_by_dollor(&telegram_token, &telegram_user_id)
                    .await?
            }
            "my" => {
                self.command_show_all_asset(&telegram_token, &telegram_user_id)
                    .await?
            }
            _ => {
                self.command_consumption_auto(&telegram_token, &telegram_user_id)
                    .await?
            }
        }

        Ok(())
    }

    // ── Shared helpers ───────────────────────────────────────────────────────

    /// Splits the raw command text after dropping its first two characters.
    ///
    /// # Arguments
    ///
    /// * `delimiter` - The string to split on after trimming the leading two characters
    ///
    /// # Returns
    ///
    /// Returns a `Vec<String>` of trimmed tokens parsed from the input text.
    /// Empty tokens are preserved when the split operation yields them.
    pub(super) fn to_preprocessed_tokens(&self, delimiter: &str) -> Vec<String> {
        let args: String = self.tele_bot_service.get_input_text();

        args.chars()
            .skip(2)
            .collect::<String>()
            .split(delimiter)
            .map(|s| s.trim().to_string())
            .collect()
    }
}
