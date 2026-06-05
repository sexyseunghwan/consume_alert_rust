use crate::common::*;

use crate::entity::{
    cash_asset, common_consume_keyword_type, currency_exchange_rate_snapshot, deposit_asset,
    saving_asset, spent_detail, telegram_room, user_payment_methods, users,
};
use crate::models::{
    cash_asset::*, crypto_resp::*, currency_exchange_rate_snapshot::*, deposit_asset::*,
    saving_asset::*, spent_detail_with_info::*, stock_resp::*, user_payment_methods::*,
};
use crate::repository::mysql_repository::*;

use super::MysqlQueryServiceImpl;

impl<R: MysqlRepository + Send + Sync> MysqlQueryServiceImpl<R> {
    pub async fn has_telegram_room_by_token_and_id(
        &self,
        room_token: &str,
        user_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        let result: Option<users::Model> = users::Entity::find()
            .inner_join(telegram_room::Entity)
            .filter(telegram_room::Column::RoomToken.eq(room_token))
            .filter(users::Column::UserId.eq(user_id))
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::has_telegram_room_by_token_and_id] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.map(|user| user.user_seq))
    }

    pub async fn find_telegram_room_seq_by_token_and_userseq(
        &self,
        room_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>> {
        let result: Option<telegram_room::Model> = telegram_room::Entity::find()
            .filter(telegram_room::Column::RoomToken.eq(room_token))
            .filter(telegram_room::Column::UserSeq.eq(user_seq))
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_telegram_room_seq_by_token_and_userseq] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.map(|room| room.room_seq))
    }

    pub async fn find_telegram_group_seq_by_token_and_userseq(
        &self,
        room_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>> {
        let result: Option<telegram_room::Model> = telegram_room::Entity::find()
            .filter(telegram_room::Column::RoomToken.eq(room_token))
            .filter(telegram_room::Column::UserSeq.eq(user_seq))
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_telegram_group_seq_by_token_and_userseq] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.and_then(|room| room.agg_group_seq))
    }

    #[allow(dead_code)]
    pub async fn find_user_id_by_seq(&self, user_seq: i64) -> anyhow::Result<Option<String>> {
        let result: Option<users::Model> = users::Entity::find()
            .filter(users::Column::UserSeq.eq(user_seq))
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_user_id_by_seq] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.map(|user| user.user_id))
    }

    #[allow(dead_code)]
    pub async fn find_latest_spent_idx(
        &self,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<Option<i64>> {
        let result: Option<spent_detail::Model> = spent_detail::Entity::find()
            .filter(spent_detail::Column::UserSeq.eq(user_seq))
            .filter(spent_detail::Column::RoomSeq.eq(room_seq))
            .order_by_desc(spent_detail::Column::SpentIdx)
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_latest_spent_idx] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.map(|row| row.spent_idx))
    }

    pub async fn find_latest_spent_detail(
        &self,
        user_seq: i64,
        room_seq: i64,
    ) -> anyhow::Result<Option<SpentDetailWithInfo>> {
        let result: Option<SpentDetailWithInfo> = spent_detail::Entity::find()
            .select_only()
            .column(spent_detail::Column::SpentIdx)
            .column(spent_detail::Column::SpentName)
            .column(spent_detail::Column::SpentMoney)
            .column(spent_detail::Column::SpentAt)
            .column(spent_detail::Column::CreatedAt)
            .column(spent_detail::Column::UserSeq)
            .column(spent_detail::Column::ConsumeKeywordTypeId)
            .column(common_consume_keyword_type::Column::ConsumeKeywordType)
            .column(spent_detail::Column::RoomSeq)
            .column(users::Column::UserId)
            .join(
                JoinType::InnerJoin,
                spent_detail::Relation::CommonConsumeKeywordType.def(),
            )
            .join(JoinType::InnerJoin, spent_detail::Relation::Users.def())
            .filter(spent_detail::Column::UserSeq.eq(user_seq))
            .filter(spent_detail::Column::RoomSeq.eq(room_seq))
            .order_by_desc(spent_detail::Column::SpentIdx)
            .into_model::<SpentDetailWithInfo>()
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_latest_spent_detail] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn find_spent_detail_with_info(
        &self,
        spent_idx: i64,
    ) -> anyhow::Result<Option<SpentDetailWithInfo>> {
        let result: Option<SpentDetailWithInfo> = spent_detail::Entity::find()
            .select_only()
            .column(spent_detail::Column::SpentIdx)
            .column(spent_detail::Column::SpentName)
            .column(spent_detail::Column::SpentMoney)
            .column(spent_detail::Column::SpentAt)
            .column(spent_detail::Column::CreatedAt)
            .column(spent_detail::Column::UserSeq)
            .column(spent_detail::Column::ConsumeKeywordTypeId)
            .column(common_consume_keyword_type::Column::ConsumeKeywordType)
            .column(spent_detail::Column::RoomSeq)
            .column(users::Column::UserId)
            .join(
                JoinType::InnerJoin,
                spent_detail::Relation::CommonConsumeKeywordType.def(),
            )
            .join(JoinType::InnerJoin, spent_detail::Relation::Users.def())
            .filter(spent_detail::Column::SpentIdx.eq(spent_idx))
            .into_model::<SpentDetailWithInfo>()
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_spent_detail_with_info] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result)
    }

    pub async fn find_user_payment_methods(
        &self,
        user_seq: i64,
        is_default: bool,
    ) -> anyhow::Result<Vec<UserPaymentMethods>> {
        let results: Vec<user_payment_methods::Model> = user_payment_methods::Entity::find()
            .filter(user_payment_methods::Column::UserSeq.eq(user_seq))
            .filter(user_payment_methods::Column::IsDefault.eq(is_default))
            .all(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_user_payment_methods] Failed to query: {:?}",
                    e
                )
            })?;

        let user_payment_methods: Vec<UserPaymentMethods> =
            results.into_iter().map(Into::into).collect();

        Ok(user_payment_methods)
    }

    pub async fn find_currency_exchange_rate_snapshot(
        &self,
        base_currency_code: &str,
        target_currency_code: &str,
    ) -> anyhow::Result<Vec<CurrencyExchangeRateSnapshot>> {
        let results: Vec<currency_exchange_rate_snapshot::Model> =
            currency_exchange_rate_snapshot::Entity::find()
                .filter(currency_exchange_rate_snapshot::Column::IsActive.eq(true))
                .filter(currency_exchange_rate_snapshot::Column::BaseCurrencyCode.eq(base_currency_code))
                .filter(currency_exchange_rate_snapshot::Column::TargetCurrencyCode.eq(target_currency_code))
                .all(self.db_conn.get_connection())
                .await
                .map_err(|e| {
                    anyhow!(
                        "[MysqlQueryServiceImpl::find_currency_exchange_rate_snapshot] Failed to query: {:?}",
                        e
                    )
                })?;

        let snapshots: Vec<CurrencyExchangeRateSnapshot> =
            results.into_iter().map(Into::into).collect();

        Ok(snapshots)
    }

    pub async fn find_deposit_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<DepositAsset>> {
        let results: Vec<deposit_asset::Model> = deposit_asset::Entity::find()
            .filter(deposit_asset::Column::UserSeq.eq(user_seq))
            .filter(deposit_asset::Column::CurrencyCode.eq(currency_code))
            .all(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_deposit_asset] Failed to query: {:?}",
                    e
                )
            })?;

        let deposit_assets: Vec<DepositAsset> = results.into_iter().map(Into::into).collect();

        Ok(deposit_assets)
    }

    pub async fn find_saving_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<SavingAsset>> {
        let results: Vec<saving_asset::Model> = saving_asset::Entity::find()
            .filter(saving_asset::Column::UserSeq.eq(user_seq))
            .filter(saving_asset::Column::CurrencyCode.eq(currency_code))
            .all(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_saving_asset] Failed to query: {:?}",
                    e
                )
            })?;

        let saving_assets: Vec<SavingAsset> = results.into_iter().map(Into::into).collect();

        Ok(saving_assets)
    }

    pub async fn find_stock_response(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<StockResp>> {
        use crate::entity::{stock, stock_asset, stock_type};
        use sea_orm::sea_query::Expr;

        let rows: Vec<StockResp> = stock_asset::Entity::find()
            .select_only()
            .column_as(
                Expr::col((stock::Entity, stock::Column::StockName)),
                "stock_name",
            )
            .column_as(
                Expr::col((stock::Entity, stock::Column::StockPrice))
                    .if_null(Decimal::ZERO)
                    .mul(Expr::col((
                        stock_asset::Entity,
                        stock_asset::Column::StockCnt,
                    ))),
                "stock_total_price",
            )
            .join(JoinType::InnerJoin, stock_asset::Relation::Stock.def())
            .join(JoinType::InnerJoin, stock::Relation::StockType.def())
            .filter(stock_type::Column::CurrencyCode.eq(currency_code))
            .filter(stock_asset::Column::UserSeq.eq(user_seq))
            .filter(stock_asset::Column::StockCnt.gt(0))
            .into_model::<StockResp>()
            .all(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_stock_response] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(rows)
    }

    pub async fn find_crypto_response(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<CryptoResp>> {
        use crate::entity::{crypto, crypto_asset};
        use sea_orm::sea_query::Expr;

        let results: Vec<CryptoResp> = crypto_asset::Entity::find()
            .select_only()
            .column_as(
                Expr::col((crypto::Entity, crypto::Column::CryptoName)),
                "crypto_name",
            )
            .column_as(
                Expr::col((crypto::Entity, crypto::Column::CryptoPrice)).mul(Expr::col((
                    crypto_asset::Entity,
                    crypto_asset::Column::CryptoCnt,
                ))),
                "crypto_total_price",
            )
            .join(JoinType::InnerJoin, crypto_asset::Relation::Crypto.def())
            .filter(crypto::Column::CurrencyCode.eq(currency_code))
            .filter(crypto_asset::Column::UserSeq.eq(user_seq))
            .filter(crypto_asset::Column::CryptoCnt.gt(0))
            .into_model::<CryptoResp>()
            .all(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_crypto_response] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(results)
    }

    pub async fn find_cash_asset(
        &self,
        user_seq: i64,
        currency_code: &str,
    ) -> anyhow::Result<Vec<CashAsset>> {
        let results: Vec<cash_asset::Model> = cash_asset::Entity::find()
            .filter(cash_asset::Column::UserSeq.eq(user_seq))
            .filter(cash_asset::Column::CurrencyCode.eq(currency_code))
            .all(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::find_cash_asset] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
