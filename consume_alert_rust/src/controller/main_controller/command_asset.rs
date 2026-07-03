use crate::common::*;

use crate::service_traits::{
    cache_service::*, elastic_query_service::*, graph_api_service::*, mysql_query_service::*,
    process_service::*, producer_service::*, redis_service::*, telebot_service::*,
};

use crate::models::{
    asset_resp::*, assets::*, cash_asset::*, crypto_resp::*, deposit_asset::*, earned_detail::*,
    per_datetime::*, saving_asset::*, stock_pie_data::*, stock_resp::*,
};

use crate::dtos::{
    StockPieDataDto
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

const SEP: &str = "--------------------------------------------";

/// Awaits `fut`, logging `ctx` alongside the error before propagating it.
async fn log_ctx<T>(
    ctx: &str,
    fut: impl std::future::Future<Output = anyhow::Result<T>>,
) -> anyhow::Result<T> {
    fut.await.inspect_err(|e| error!("{}: {:#}", ctx, e))
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
    /* 해당 자산의 달러/원화 가치 모두 저장하고 있음. */
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

    let sections: &[(&str, &str)] = &[
        ("Deposit", "예금성 자산"),
        ("Saving", "적금성 자산"),
        ("Stock", "주식성 자산"),
        ("Crypto", "크립토성 자산"),
        ("Cash", "현금성 자산"),
    ];
    let mut msg: String = format!(
        "총자산 = {}₩ // {:.2}$\n",
        format_decimal_with_commas(grand_krw, 0),
        grand_usd.round_dp(2),
    );

    for (key, label) in sections {
        msg.push_str(&format!("{}\n[{}]\n", SEP, label));
        let assets: &[AssetResp] = asset_map.get(*key).map(Vec::as_slice).unwrap_or(&[]);

        let mut section_krw: Decimal = Decimal::ZERO;
        let mut section_usd: Decimal = Decimal::ZERO;

        if assets.is_empty() {
            msg.push_str("  (없음)\n");
        } else {
            for asset in assets {
                msg.push_str(&format!(
                    "*  {} : {}₩ ({:.2}$)\n",
                    asset.asset_name(),
                    format_decimal_with_commas(asset.asset_krw, 0),
                    asset.asset_usd.round_dp(2),
                ));
                section_krw += asset.asset_krw;
                section_usd += asset.asset_usd;
            }
        }

        msg.push_str(&format!(
            "{} 총계 : {}₩ ({:.2}$)\n",
            label,
            format_decimal_with_commas(section_krw, 0),
            section_usd.round_dp(2),
        ));
    }

    msg.push_str(SEP);
    msg
}

fn build_stock_message(
    stock_resp_details: &[StockRespDetail],
    total_stock_amount_krw: Decimal,
    stock_avg_purchase_price_krw: Decimal,
    rates: ExchangeRates,
) -> String {
    let total_stock_amount_usd: Decimal = total_stock_amount_krw * rates.krw_to_usd;

    let mut msg: String = format!("{}\n[주식 포트폴리오]\n", SEP);

    if stock_resp_details.is_empty() {
        msg.push_str("  (없음)\n");
    } else {
        for stock in stock_resp_details {
            msg.push_str(&format!(
                "*  {} : \n      {}₩ ({:.2}$) \n            ROI: {:.3}%\n            PROFIT(₩): {}\n",
                stock.stock_alias(),
                format_decimal_with_commas(stock.stock_total_price_krw, 0),
                stock.stock_total_price_usd.round_dp(2),
                stock.stock_roi,
                format_decimal_with_commas(stock.stock_invest_profit_krw, 0)
            ));
        }
    }

    let total_stock_profit: Decimal = total_stock_amount_krw - stock_avg_purchase_price_krw;
    let total_stock_roi: Decimal = total_stock_profit / stock_avg_purchase_price_krw * Decimal::from(100);

    msg.push_str(&format!(
        "{}\n총 주식: \n      {}₩ ({:.2}$)\n            ROI: {:.3}%\n            PROFIT(₩): {}\n",
        SEP,
        format_decimal_with_commas(total_stock_amount_krw, 0),
        total_stock_amount_usd.round_dp(2),
        total_stock_roi.round_dp(2),
        format_decimal_with_commas(total_stock_profit, 0)
    ));

    msg.push_str(SEP);
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
    /// Resolves the caller's user sequence and telegram room sequence together.
    async fn resolve_identity(
        &self,
        telegram_token: &str,
        telegram_user_id: &str,
    ) -> anyhow::Result<(i64, i64)> {
        let user_seq: i64 = self
            .resolve_user_seq(telegram_token, telegram_user_id)
            .await?;

        let room_seq: i64 = self
            .resolve_telegram_room_seq(user_seq, telegram_token, telegram_user_id)
            .await?;

        Ok((user_seq, room_seq))
    }

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

        let (user_seq, room_seq) = self
            .resolve_identity(telegram_token, telegram_user_id)
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

        let usd_amount: f64 = log_ctx(
            "[main_controller::command_earend_detail_by_won] Failed to convert KRW to USD",
            krw_to_usd(earned_money),
        )
        .await?;

        let earned_money_dollor: Decimal = Decimal::try_from(usd_amount).map_err(|e| {
            anyhow!(
                "[main_controller::command_earend_detail_by_won] Failed to convert f64 to Decimal: {:#}",
                e
            )
        })?;

        let earned_detail: EarnedDetail = EarnedDetail {
            earned_name: earned_name.clone(),
            earned_money,
            earned_money_dollor,
            earned_at: Utc::now().into(),
            user_seq,
            room_seq,
        };

        log_ctx(
            "[main_controller::command_earend_detail_by_won] Failed to insert to MySQL",
            self.mysql_query_service
                .input_earned_detail_with_transaction(&earned_detail),
        )
        .await?;

        let confirm_msg: String = format!(
            "Earned detail saved!\nName  : {}\nKRW   : {} 원\nUSD   : $ {:.2}",
            earned_name,
            earned_money.to_formatted_string(&Locale::en),
            usd_amount,
        );

        log_ctx(
            "[main_controller::command_earend_detail_by_won] Failed to send Telegram message",
            self.tele_bot_service.input_message_confirm(&confirm_msg),
        )
        .await?;

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

        let (user_seq, room_seq) = self
            .resolve_identity(telegram_token, telegram_user_id)
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

        let earned_money: i64 = log_ctx(
            "[main_controller::command_earend_detail_by_dollor] Failed to convert USD to KRW",
            usd_to_krw(usd_amount),
        )
        .await?;

        let earned_money_dollor: Decimal = Decimal::try_from(usd_amount).map_err(|e| {
            anyhow!(
                "[main_controller::command_earend_detail_by_dollor] Failed to convert f64 to Decimal: {:#}",
                e
            )
        })?;

        let earned_detail: EarnedDetail = EarnedDetail {
            earned_name: earned_name.clone(),
            earned_money,
            earned_money_dollor,
            earned_at: Utc::now().into(),
            user_seq,
            room_seq,
        };

        log_ctx(
            "[main_controller::command_earend_detail_by_dollor] Failed to insert to MySQL",
            self.mysql_query_service
                .input_earned_detail_with_transaction(&earned_detail),
        )
        .await?;

        let confirm_msg: String = format!(
            "Earned detail saved!\nName  : {}\nUSD   : $ {:.2}\nKRW   : {} 원",
            earned_name,
            usd_amount,
            earned_money.to_formatted_string(&Locale::en),
        );

        log_ctx(
            "[main_controller::command_earend_detail_by_dollor] Failed to send Telegram message",
            self.tele_bot_service.input_message_confirm(&confirm_msg),
        )
        .await?;

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

                let mut totals: AssetTotals = AssetTotals {
                    krw: Decimal::ZERO,
                    usd: Decimal::ZERO,
                };
                let rates: ExchangeRates = ExchangeRates {
                    usd_to_krw,
                    krw_to_usd,
                };

                let mut asset_map: HashMap<String, Vec<AssetResp>> = HashMap::new();
                let mut stock_list: Vec<StockResp> = Vec::new();

                let mut total_stock_amount_krw: Decimal = Decimal::ZERO;
                //let mut total_stock_amount_usd: Decimal = Decimal::ZERO;

                for currency_code in &["KRW", "USD"] {
                    let is_krw: bool = *currency_code == "KRW";

                    let deposits: Vec<DepositAsset> = log_ctx(
                        "[command_show_all_asset] deposits",
                        self.mysql_query_service
                            .find_deposit_asset(user_seq, currency_code),
                    )
                    .await?;

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

                    let savings: Vec<SavingAsset> = log_ctx(
                        "[command_show_all_asset] savings",
                        self.mysql_query_service
                            .find_saving_asset(user_seq, currency_code),
                    )
                    .await?;

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

                    let stock_resps: Vec<StockResp> = log_ctx(
                        "[command_show_all_asset] stocks",
                        self.mysql_query_service
                            .find_stock_response(user_seq, currency_code),
                    )
                    .await?;

                    for s in &stock_resps {
                        let stock_amount: Decimal = s.stock_price * Decimal::from(*s.stock_cnt());
                        push_asset(
                            &mut asset_map,
                            &mut totals,
                            "Stock",
                            s.stock_alias().to_string(), // 이거 왜 안되는거냐
                            stock_amount,
                            is_krw,
                            rates,
                        );

                        if is_krw {
                            //total_stock_amount_usd += stock_amount * krw_to_usd;
                            total_stock_amount_krw += stock_amount;
                        } else {
                            //total_stock_amount_usd += stock_amount;
                            total_stock_amount_krw += stock_amount * usd_to_krw;
                        }
                        stock_list.push(s.clone());
                    }

                    let cryptos: Vec<CryptoResp> = log_ctx(
                        "[command_show_all_asset] cryptos",
                        self.mysql_query_service
                            .find_crypto_response(user_seq, currency_code),
                    )
                    .await?;

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

                    let cashes: Vec<CashAsset> = log_ctx(
                        "[command_show_all_asset] cashes",
                        self.mysql_query_service.find_cash_asset(user_seq, currency_code),
                    )
                    .await?;
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

                log_ctx(
                    "[command_show_all_asset] Failed to send message",
                    self.tele_bot_service.input_message_confirm(&msg),
                )
                .await?;

                let total_asset_amount_krw: Decimal = totals.krw + (totals.usd * usd_to_krw);
                let assets: Assets = Assets::new(total_asset_amount_krw, asset_map);

                let pie_image_bytes: Vec<u8> = log_ctx(
                    "[command_show_all_asset] Failed to get asset pie image",
                    self.graph_api_service.find_python_matplot_asset_pie(assets),
                )
                .await?;

                log_ctx(
                    "[command_show_all_asset] Failed to send asset pie image",
                    self.tele_bot_service
                        .input_photo_from_bytes(pie_image_bytes, "asset_pie.png"),
                )
                .await?;
                
                /* 이걸 기준으로 봐야함!! */
                let stock_resp_details: Vec<StockRespDetail> = stock_list
                    .iter()
                    .map(|stock| {
                        stock.convert_to_stock_resp_detail(
                            total_stock_amount_krw,
                            stock.currency_code().to_string(),
                            usd_to_krw,
                            krw_to_usd,
                        )
                    })
                    .collect();

                let stock_avg_purchase_price_krw: Decimal = stock_resp_details
                    .iter()
                    .map(|stock| stock.avg_purchase_price_krw)
                    .sum();
                
                let stock_msg: String = build_stock_message(
                    &stock_resp_details,
                    total_stock_amount_krw,
                    stock_avg_purchase_price_krw,
                    rates,
                );

                log_ctx(
                    "[command_show_all_asset] Failed to send stock message",
                    self.tele_bot_service.input_message_confirm(&stock_msg),
                )
                .await?;
                
                let etc_threshold: Decimal = Decimal::new(3, 2);
                let mut stock_pie_data_dtos: Vec<StockPieDataDto> = Vec::new();
                let mut etc_amount_krw: Decimal = Decimal::ZERO;

                for resp in &stock_resp_details {
                    if resp.stock_portfolio_weight <= etc_threshold {
                        etc_amount_krw += *resp.stock_total_price_krw();
                    } else {
                        stock_pie_data_dtos.push(StockPieDataDto {
                            stock_alias: resp.stock_alias.clone(),
                            stock_amount_krw: *resp.stock_total_price_krw(),
                        });
                    }
                }

                if etc_amount_krw != Decimal::ZERO {
                    stock_pie_data_dtos.push(StockPieDataDto {
                        stock_alias: "ETC".to_string(),
                        stock_amount_krw: etc_amount_krw,
                    });
                }
                
                let stock_pie_data: StockPieData = StockPieData::new(
                    stock_pie_data_dtos.iter().map(|s| s.stock_alias().to_string()).collect(),
                    stock_pie_data_dtos.iter().map(|s| *s.stock_amount_krw()).collect(),
                    total_stock_amount_krw,
                );

                let stock_pie_bytes: Vec<u8> = log_ctx(
                    "[command_show_all_asset] Failed to get stock pie image",
                    self.graph_api_service
                        .find_python_matplot_stock_pie(stock_pie_data),
                )
                .await?;

                log_ctx(
                    "[command_show_all_asset] Failed to send stock pie image",
                    self.tele_bot_service
                        .input_photo_from_bytes(stock_pie_bytes, "stock_pie.png"),
                )
                .await?;
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
