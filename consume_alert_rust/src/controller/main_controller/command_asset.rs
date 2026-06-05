use crate::common::*;

use crate::service_traits::cache_service::*;
use crate::service_traits::elastic_query_service::*;
use crate::service_traits::graph_api_service::*;
use crate::service_traits::mysql_query_service::*;
use crate::service_traits::process_service::*;
use crate::service_traits::producer_service::*;
use crate::service_traits::redis_service::*;
use crate::service_traits::telebot_service::*;

use crate::models::{
    asset_resp::*, cash_asset::*, crypto_resp::*, deposit_asset::*, earned_detail::*,
    per_datetime::*, saving_asset::*, stock_resp::*,
};

use crate::utils_modules::{currency_utils::*, io_utils::*, numeric_utils::*, time_utils::*};

use super::MainController;

#[derive(Clone, Copy)]
struct ExchangeRates {
    usd_to_krw: Decimal,
    krw_to_usd: Decimal,
}

struct AssetTotals {
    krw: Decimal,
    usd: Decimal,
}

fn push_asset(
    map: &mut HashMap<String, Vec<AssetResp>>,
    totals: &mut AssetTotals,
    asset_type: &str,
    name: String,
    amount: Decimal,
    is_krw: bool,
    rates: ExchangeRates,
) {
    let (krw, usd) = if is_krw {
        totals.krw += amount;
        (amount, amount * rates.krw_to_usd)
    } else {
        totals.usd += amount;
        (amount * rates.usd_to_krw, amount)
    };
    map.entry(asset_type.to_string())
        .or_default()
        .push(AssetResp::new(asset_type.to_string(), name, krw, usd));
}

fn build_asset_message(
    asset_map: &HashMap<String, Vec<AssetResp>>,
    totals: &AssetTotals,
    rates: ExchangeRates,
) -> String {
    let grand_krw: Decimal = totals.krw + (totals.usd * rates.usd_to_krw);
    let grand_usd: Decimal = totals.usd + (totals.krw * rates.krw_to_usd);
    let grand_krw_i64: i64 = grand_krw.round().to_string().parse().unwrap_or(0);

    let sep: &str = "--------------------------------------------";
    let sections: &[(&str, &str)] = &[
        ("Deposit", "예금성 자산"),
        ("Saving", "적금성 자산"),
        ("Stock", "주식성 자산"),
        ("Crypto", "크립토성 자산"),
        ("Cash", "현금성 자산"),
    ];
    let empty_vec: Vec<AssetResp> = Vec::new();

    let mut msg: String = format!(
        "총자산 = {}₩ // {:.2}$\n",
        grand_krw_i64.to_formatted_string(&Locale::en),
        grand_usd.round_dp(2),
    );

    for (key, label) in sections {
        msg.push_str(&format!("{}\n[{}]\n", sep, label));
        let assets: &Vec<AssetResp> = asset_map.get(*key).unwrap_or(&empty_vec);

        let mut section_krw: Decimal = Decimal::ZERO;
        let mut section_usd: Decimal = Decimal::ZERO;

        if assets.is_empty() {
            msg.push_str("  (없음)\n");
        } else {
            for asset in assets {
                let krw_i64: i64 = asset.asset_krw.round().to_string().parse().unwrap_or(0);
                msg.push_str(&format!(
                    "*  {} : {}₩ ({:.2}$)\n",
                    asset.asset_name(),
                    krw_i64.to_formatted_string(&Locale::en),
                    asset.asset_usd.round_dp(2),
                ));
                section_krw += asset.asset_krw;
                section_usd += asset.asset_usd;
            }
        }

        let section_krw_i64: i64 = section_krw.round().to_string().parse().unwrap_or(0);
        msg.push_str(&format!(
            "{} 총계 : {}₩ ({:.2}$)\n",
            label,
            section_krw_i64.to_formatted_string(&Locale::en),
            section_usd.round_dp(2),
        ));
    }

    msg.push_str(sep);
    msg
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

        let room_seq: i64 = self
            .resolve_telegram_room_seq(user_seq, telegram_token, telegram_user_id)
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
            room_seq,
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

        let room_seq: i64 = self
            .resolve_telegram_room_seq(user_seq, telegram_token, telegram_user_id)
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
            room_seq,
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

    #[allow(dead_code)]
    pub(super) async fn command_earend_detail_per_mon(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.to_preprocessed_tokens(" ");

        let _permon_datetime: PerDatetime = match args.len() {
            1 => {
                let date_start: DateTime<Utc> = find_current_kor_naivedate_first_date()?;
                let date_end: DateTime<Utc> = find_lastday_naivedate(date_start)?;

                self.process_service
                    .find_nmonth_to_current_date(date_start, date_end, -1)?
            }
            2 if args
                .get(1)
                .is_some_and(|d| is_valid_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) =>
            {
                let parts: Vec<&str> = args[1].split('.').collect();
                let year: i32 = parts
                    .first()
                    .ok_or_else(|| anyhow!("[command_consumption_per_mon] Missing year"))?
                    .parse()?;
                let month: u32 = parts
                    .get(1)
                    .ok_or_else(|| anyhow!("[command_consumption_per_mon] Missing month"))?
                    .parse()?;
                let date_start: DateTime<Utc> = find_naivedate(year, month, 1)?;
                let date_end: DateTime<Utc> = find_lastday_naivedate(date_start)?;
                self.process_service
                    .find_nmonth_to_current_date(date_start, date_end, -1)?
            }
            _ => {
                self.tele_bot_service
                    .input_message_confirm(
                        "Invalid date format. Please use format YYYY.MM like em 2023.07 or em",
                    )
                    .await?;
                return Err(anyhow!(
                    "[command_consumption_per_mon] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        let user_seq: i64 = self
            .resolve_user_seq(telegram_token, telegram_user_id)
            .await?;

        let _room_seq: i64 = self
            .resolve_telegram_room_seq(user_seq, telegram_token, telegram_user_id)
            .await?;

        Ok(())
    }

    pub(super) async fn command_show_all_asset(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<()> {
        let args: Vec<String> = self.to_preprocessed_tokens(" ");

        match args.len() {
            1 => {
                let user_seq: i64 = self
                    .resolve_user_seq(telegram_token, telegram_user_id)
                    .await?;

                let usd_to_krw: Decimal =
                    fetch_exchange_rate(self.mysql_query_service.as_ref(), "USD", "KRW").await?;
                let krw_to_usd: Decimal =
                    fetch_exchange_rate(self.mysql_query_service.as_ref(), "KRW", "USD").await?;

                let mut totals = AssetTotals {
                    krw: Decimal::ZERO,
                    usd: Decimal::ZERO,
                };
                let rates = ExchangeRates {
                    usd_to_krw,
                    krw_to_usd,
                };
                let mut asset_map: HashMap<String, Vec<AssetResp>> = HashMap::new();

                for currency_code in &["KRW", "USD"] {
                    let is_krw: bool = *currency_code == "KRW";

                    let deposits: Vec<DepositAsset> = self
                        .mysql_query_service
                        .find_deposit_asset(user_seq, currency_code)
                        .await
                        .inspect_err(|e| error!("[command_show_all_asset] deposits: {:#}", e))?;
                    for d in &deposits {
                        push_asset(
                            &mut asset_map,
                            &mut totals,
                            "Deposit",
                            d.deposit_name().to_string(),
                            *d.deposit_amount(),
                            is_krw,
                            rates,
                        );
                    }

                    let savings: Vec<SavingAsset> = self
                        .mysql_query_service
                        .find_saving_asset(user_seq, currency_code)
                        .await
                        .inspect_err(|e| error!("[command_show_all_asset] savings: {:#}", e))?;
                    for s in &savings {
                        push_asset(
                            &mut asset_map,
                            &mut totals,
                            "Saving",
                            s.saving_name().to_string(),
                            *s.accum_saving_amount(),
                            is_krw,
                            rates,
                        );
                    }

                    let stocks: Vec<StockResp> = self
                        .mysql_query_service
                        .find_stock_response(user_seq, currency_code)
                        .await
                        .inspect_err(|e| error!("[command_show_all_asset] stocks: {:#}", e))?;
                    for s in &stocks {
                        push_asset(
                            &mut asset_map,
                            &mut totals,
                            "Stock",
                            s.stock_name().to_string(),
                            *s.stock_total_price(),
                            is_krw,
                            rates,
                        );
                    }

                    let cryptos: Vec<CryptoResp> = self
                        .mysql_query_service
                        .find_crypto_response(user_seq, currency_code)
                        .await
                        .inspect_err(|e| error!("[command_show_all_asset] cryptos: {:#}", e))?;
                    for c in &cryptos {
                        push_asset(
                            &mut asset_map,
                            &mut totals,
                            "Crypto",
                            c.crypto_name().to_string(),
                            *c.crypto_total_price(),
                            is_krw,
                            rates,
                        );
                    }

                    let cashes: Vec<CashAsset> = self
                        .mysql_query_service
                        .find_cash_asset(user_seq, currency_code)
                        .await
                        .inspect_err(|e| error!("[command_show_all_asset] cashes: {:#}", e))?;
                    for c in &cashes {
                        push_asset(
                            &mut asset_map,
                            &mut totals,
                            "Cash",
                            c.cash_name().to_string(),
                            *c.cash(),
                            is_krw,
                            rates,
                        );
                    }
                }

                let msg: String = build_asset_message(&asset_map, &totals, rates);

                self.tele_bot_service
                    .input_message_confirm(&msg)
                    .await
                    .inspect_err(|e| {
                        error!("[command_show_all_asset] Failed to send message: {:#}", e)
                    })?;
            }
            _ => {
                self.tele_bot_service
                    .input_message_confirm("Invalid date format. Please use format `my`")
                    .await?;
                return Err(anyhow!(
                    "[command_show_all_asset] Invalid parameter: {:?}",
                    self.tele_bot_service.get_input_text()
                ));
            }
        };

        Ok(())
    }
}
