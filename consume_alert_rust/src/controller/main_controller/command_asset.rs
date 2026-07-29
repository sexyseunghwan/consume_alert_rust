use crate::common::*;

use crate::service_traits::{
    cache_service::*, elastic_query_service::*, graph_api_service::*, mysql_query_service::*,
    process_service::*, producer_service::*, redis_service::*, telebot_service::*,
};

use crate::models::{
    asset_collection::*, asset_resp::*, assets::*, cash_asset::*, crypto_resp::*, deposit_asset::*,
    earned_detail::*, file_info::*, per_datetime::*, saving_asset::*, stock_pie_data::*,
    stock_resp::*, to_python_graph_line::*, user_asset_snapshot_summary::*,
};

use crate::utils_modules::{
    common_function::log_ctx, currency_utils::*, io_utils::*, numeric_utils::*, time_utils::*,
};

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

    /// Fetches the USD/KRW and KRW/USD exchange rates used to normalize every asset amount.
    async fn fetch_exchange_rates(&self) -> anyhow::Result<ExchangeRates> {
        let usd_to_krw: Decimal =
            fetch_exchange_rate(self.mysql_query_service.as_ref(), "USD", "KRW").await?;
        let krw_to_usd: Decimal =
            fetch_exchange_rate(self.mysql_query_service.as_ref(), "KRW", "USD").await?;

        Ok(ExchangeRates {
            usd_to_krw,
            krw_to_usd,
        })
    }

    /// Gathers every asset type (deposit/saving/stock/crypto/cash) in both KRW and USD for `user_seq`.
    async fn collect_all_assets(
        &self,
        user_seq: i64,
        rates: ExchangeRates,
    ) -> anyhow::Result<AssetCollection> {
        let mut totals: AssetTotals = AssetTotals {
            krw: Decimal::ZERO,
            usd: Decimal::ZERO,
        };

        let mut asset_map: HashMap<String, Vec<AssetResp>> = HashMap::new();
        let mut stock_list: Vec<StockResp> = Vec::new();

        let mut total_stock_amount_krw: Decimal = Decimal::ZERO;

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
                    s.stock_alias().to_string(),
                    stock_amount,
                    is_krw,
                    rates,
                );

                if is_krw {
                    total_stock_amount_krw += stock_amount;
                } else {
                    total_stock_amount_krw += stock_amount * rates.usd_to_krw;
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
                self.mysql_query_service
                    .find_cash_asset(user_seq, currency_code),
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

        Ok(AssetCollection {
            asset_map,
            totals,
            stock_list,
            total_stock_amount_krw,
        })
    }

    /* 1. 총자산 요약 정보 */
    async fn send_asset_summary_message(
        &self,
        asset_map: &HashMap<String, Vec<AssetResp>>,
        totals: &AssetTotals,
        rates: ExchangeRates,
    ) -> anyhow::Result<()> {
        let msg: String = build_asset_message(asset_map, totals, rates);

        log_ctx(
            "[command_show_all_asset] Failed to send message",
            self.tele_bot_service.input_message_confirm(&msg),
        )
        .await
    }

    /* 2. 총자산 요약 정보 - 파이 그래프 */
    async fn send_asset_summary_pie(
        &self,
        asset_map: HashMap<String, Vec<AssetResp>>,
        total_asset_amount_krw: Decimal,
    ) -> anyhow::Result<()> {
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
        .await
    }

    /* 3. 주식 포트폴리오 정보 */
    async fn send_stock_summary_message(
        &self,
        stock_resp_details: &[StockRespDetail],
        total_stock_amount_krw: Decimal,
        stock_avg_purchase_price_krw: Decimal,
        rates: ExchangeRates,
    ) -> anyhow::Result<()> {
        let stock_msg: String = build_stock_message(
            stock_resp_details,
            total_stock_amount_krw,
            stock_avg_purchase_price_krw,
            rates,
        );

        log_ctx(
            "[command_show_all_asset] Failed to send stock message",
            self.tele_bot_service.input_message_confirm(&stock_msg),
        )
        .await
    }

    /* 4. 주식 포트폴리오 정보 - 파이 그래프 */
    async fn send_stock_pie(&self, stock_pie_data: StockPieData) -> anyhow::Result<()> {
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
        .await
    }

    /// Fetches asset snapshots in `[start, end)` for `user_seq`, logging the KST/UTC range for `label`.
    async fn fetch_asset_snapshot_range(
        &self,
        user_seq: i64,
        label: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<UserAssetSnapshotSummary>> {
        info!(
            "[command_show_all_asset] {} range - KST: [{}, {}) / UTC: [{}, {})",
            label,
            start.with_timezone(&Seoul),
            end.with_timezone(&Seoul),
            start,
            end
        );

        log_ctx(
            &format!("[command_show_all_asset] {} asset snapshot summary", label),
            self.mysql_query_service
                .find_user_asset_snapshot_summary(user_seq, start, end),
        )
        .await
    }

    /// Fetches one period's snapshots, renders the Python line graph, and wraps it as a `FileInfo`.
    async fn render_asset_history_graph(
        &self,
        user_seq: i64,
        label: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<FileInfo> {
        let snapshots: Vec<UserAssetSnapshotSummary> = self
            .fetch_asset_snapshot_range(user_seq, label, start, end)
            .await?;

        let python_graph: ToPythonGraphLine =
            ToPythonGraphLine::new("", start, end, 0.0, &snapshots, LineAggregation::Raw)?;

        let graph_bytes: Vec<u8> = self
            .graph_api_service
            .find_python_matplot_asset_history(&python_graph)
            .await?;

        Ok(FileInfo::new(format!("{label}_asset_history"), graph_bytes))
    }

    /* 5. 총자산 변동 그래프 (일/주/월/분기/반기/년)
       자산 스냅샷은 UTC로 저장되어 있지만, 조회 구간은 사용자가 인식하는
       KST(UTC+9) 날짜 경계를 기준으로 계산한 뒤 UTC로 변환해야 한다.
       끝점 유실을 막기 위해 `start_at <= aggregated_at < end_at` 반개구간으로 조회한다.

       ex) KST 오늘 2026-07-10 하루
           KST: 2026-07-10 00:00:00 +09:00 ~ 2026-07-11 00:00:00 +09:00
           UTC: 2026-07-09 15:00:00 UTC     ~ 2026-07-10 15:00:00 UTC */
    async fn send_asset_history_graphs(&self, user_seq: i64) -> anyhow::Result<()> {
        let now_kst: DateTime<chrono_tz::Tz> = Utc::now().with_timezone(&Seoul);
        info!("[command_show_all_asset] now_kst: {:?}", now_kst);

        /* KST 오늘 날짜 (달력 상의 날짜이므로 월 연산은 반드시 이 날짜를 기준으로 수행한다) */
        let kst_today: NaiveDate = now_kst.date_naive();

        /* 내일 KST 00:00:00 을 UTC 로 환산한 값 (반개구간의 끝점, 모든 기간이 공통으로 사용) */
        let tomorrow_start_utc: DateTime<Utc> =
            kst_midnight_to_utc(kst_today.succ_opt().ok_or_else(|| {
                anyhow!("[command_show_all_asset] Date overflow computing tomorrow")
            })?)?;

        let periods: [(&str, DateTime<Utc>); 6] = [
            ("daily", kst_midnight_to_utc(kst_today)?),
            ("weekly", kst_days_ago(kst_today, 6)?),
            ("monthly", kst_months_ago(kst_today, 1)?),
            ("quarterly", kst_months_ago(kst_today, 3)?),
            ("half_yearly", kst_months_ago(kst_today, 6)?),
            ("yearly", kst_months_ago(kst_today, 12)?),
        ];

        let mut img_files: Vec<FileInfo> = Vec::with_capacity(periods.len());

        for (label, start_utc) in periods {
            let file: FileInfo = self
                .render_asset_history_graph(user_seq, label, start_utc, tomorrow_start_utc)
                .await?;
            img_files.push(file);
        }

        self.tele_bot_service.input_photo_confirm(img_files).await?;

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

                let rates: ExchangeRates = self.fetch_exchange_rates().await?;
                let collection: AssetCollection = self.collect_all_assets(user_seq, rates).await?;

                /* 1. 총자산 요약 정보 */
                self.send_asset_summary_message(&collection.asset_map, &collection.totals, rates)
                    .await?;

                /* 2. 총자산 요약 정보 - 파이 그래프 */
                let total_asset_amount_krw: Decimal =
                    collection.totals.krw + (collection.totals.usd * rates.usd_to_krw);
                self.send_asset_summary_pie(collection.asset_map, total_asset_amount_krw)
                    .await?;

                /* 3. 주식 포트폴리오 정보 */
                let stock_resp_details: Vec<StockRespDetail> = build_stock_details(
                    &collection.stock_list,
                    collection.total_stock_amount_krw,
                    rates.usd_to_krw,
                    rates.krw_to_usd,
                );
                let stock_avg_purchase_price_krw: Decimal = stock_resp_details
                    .iter()
                    .map(|stock| stock.avg_purchase_price_krw)
                    .sum();
                self.send_stock_summary_message(
                    &stock_resp_details,
                    collection.total_stock_amount_krw,
                    stock_avg_purchase_price_krw,
                    rates,
                )
                .await?;

                /* 4. 주식 포트폴리오 정보 - 파이 그래프 */
                let stock_pie_data: StockPieData =
                    build_stock_pie_data(&stock_resp_details, collection.total_stock_amount_krw);
                self.send_stock_pie(stock_pie_data).await?;

                /* 5. 총자산 변동 그래프 */
                self.send_asset_history_graphs(user_seq).await?;
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
