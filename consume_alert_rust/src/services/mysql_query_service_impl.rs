use crate::common::*;

use crate::entity::{
    common_consume_keyword_type, spent_detail, telegram_room, user_payment_methods, users,
};
use crate::models::spent_detail::*;
use crate::models::spent_detail_with_info::*;
use crate::models::user_payment_methods::*;
use crate::repository::mysql_repository::*;

use crate::service_traits::mysql_query_service::*;

#[derive(Debug, Getters, Clone, new)]
pub struct MysqlQueryServiceImpl<R: MysqlRepository> {
    db_conn: R,
}

impl<R: MysqlRepository + Send + Sync> MysqlQueryServiceImpl<R> {}

#[async_trait]
impl<R: MysqlRepository + Send + Sync> MysqlQueryService for MysqlQueryServiceImpl<R> {
    /// Converts a `SpentDetail` domain model and inserts it into the database within a transaction.
    ///
    /// # Arguments
    ///
    /// * `spent_detail` - The spending record to persist
    ///
    /// # Returns
    ///
    /// Returns `Ok(i64)` containing the auto-incremented `spent_idx` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion to ActiveModel fails or the database transaction fails.
    async fn insert_prodt_detail_with_transaction(
        &self,
        spent_detail: &SpentDetail,
    ) -> anyhow::Result<i64> {
        let active_model: spent_detail::ActiveModel = spent_detail
            .convert_spent_detail_to_active_model()
            .context("[insert_prodt_detail_with_transaction] Failed to convert to ActiveModel")?;

        self.db_conn
            .insert_spent_detail_with_transaction(active_model)
            .await
    }

    /// Converts multiple `SpentDetail` records and inserts them sequentially within a single transaction.
    ///
    /// # Arguments
    ///
    /// * `spent_details` - Slice of spending records to persist in order
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<i64>)` containing the auto-incremented `spent_idx` values in insertion order.
    ///
    /// # Errors
    ///
    /// Returns an error if any conversion or database operation fails; the transaction is rolled back.
    async fn insert_prodt_details_with_transaction(
        &self,
        spent_details: &[SpentDetail],
    ) -> anyhow::Result<Vec<i64>> {
        // Convert domain models to SeaORM ActiveModels in the same order as the input slice.
        // Order must be preserved here so that the index returned by the repository
        // corresponds to the correct SpentDetail at the same position.
        let mut active_models: Vec<spent_detail::ActiveModel> =
            Vec::with_capacity(spent_details.len());

        for (position, detail) in spent_details.iter().enumerate() {
            let active_model: spent_detail::ActiveModel =
                detail.convert_spent_detail_to_active_model().map_err(|e| {
                    anyhow!(
                        "[MysqlQueryServiceImpl::insert_prodt_details_with_transaction] \
                     Failed to convert SpentDetail at position {} to ActiveModel: {:?}",
                        position,
                        e
                    )
                })?;

            active_models.push(active_model);
        }

        // Delegate to the repository, which inserts each record sequentially inside
        // one transaction and returns `spent_idx` values in the same order as `active_models`.
        self.db_conn
            .insert_spent_details_with_transaction(active_models)
            .await
    }

    /// Checks whether a Telegram room exists for the given token and user ID, returning the user sequence if found.
    ///
    /// # Arguments
    ///
    /// * `room_token` - The Telegram room token to match
    /// * `user_id` - The Telegram user ID to match
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(i64))` with the `user_seq` if a matching record is found, or `Ok(None)` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn exists_telegram_room_by_token_and_id(
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
                    "[MysqlQueryServiceImpl::exists_telegram_room_by_token_and_id] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.map(|user| user.user_seq))
    }

    /// Retrieves the room sequence number for a Telegram room identified by token and user sequence.
    ///
    /// # Arguments
    ///
    /// * `room_token` - The Telegram room token to match
    /// * `user_seq` - The user sequence number to match
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(i64))` with the `room_seq` if found, or `Ok(None)` if no match exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn get_telegram_room_seq_by_token_and_userseq(
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
                    "[MysqlQueryServiceImpl::get_telegram_room_seq_by_token_and_userseq] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.map(|room| room.room_seq))
    }

    /// Retrieves the user ID string for the given user sequence number.
    ///
    /// # Arguments
    ///
    /// * `user_seq` - The unique sequence number identifying the user
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(String))` with the user ID if found, or `Ok(None)` if no record exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn get_user_id_by_seq(&self, user_seq: i64) -> anyhow::Result<Option<String>> {
        let result: Option<users::Model> = users::Entity::find()
            .filter(users::Column::UserSeq.eq(user_seq))
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::get_user_id_by_seq] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.map(|user| user.user_id))
    }

    /// Retrieves the most recently inserted `spent_idx` for the given user and room.
    ///
    /// # Arguments
    ///
    /// * `user_seq` - The unique sequence number identifying the user
    /// * `room_seq` - The unique sequence number identifying the Telegram room
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(i64))` with the latest `spent_idx`, or `Ok(None)` if no records exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn get_latest_spent_idx(
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
                    "[MysqlQueryServiceImpl::get_latest_spent_idx] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result.map(|row| row.spent_idx))
    }

    /// Retrieves a spending record joined with keyword type and user information by its primary key.
    ///
    /// # Arguments
    ///
    /// * `spent_idx` - The primary key of the spending record
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(SpentDetailWithInfo))` if found, or `Ok(None)` if no record exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn get_spent_detail_with_info(
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
                    "[MysqlQueryServiceImpl::get_spent_detail_with_info] Failed to query: {:?}",
                    e
                )
            })?;

        Ok(result)
    }

    /// Deletes the spending record identified by `spent_idx` within a database transaction.
    ///
    /// # Arguments
    ///
    /// * `spent_idx` - The primary key of the record to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the database transaction fails; the deletion is rolled back.
    async fn delete_spent_detail_with_transaction(&self, spent_idx: i64) -> anyhow::Result<()> {
        self.db_conn
            .delete_spent_detail_with_transaction(spent_idx)
            .await
    }

    /// Retrieves the payment methods registered by the specified user, optionally filtering by default flag.
    ///
    /// # Arguments
    ///
    /// * `user_seq` - The unique sequence number identifying the user
    /// * `is_default` - If `true`, returns only the default payment method; otherwise returns all
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<UserPaymentMethods>)` with the matching payment methods.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn get_user_payment_methods(
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
                    "[MysqlQueryServiceImpl::get_user_payment_methods] Failed to query: {:?}",
                    e
                )
            })?;

        let user_payment_methods: Vec<UserPaymentMethods> = results
            .into_iter()
            .map(|model| {
                UserPaymentMethods::new(
                    model.payment_method_id,
                    model.payment_type_cd,
                    model.payment_category_cd,
                    model.card_id,
                    model.card_alias,
                    model.is_active,
                    DateTime::from_naive_utc_and_offset(model.created_at, Utc),
                    model
                        .updated_at
                        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
                    model.created_by,
                    model.updated_by,
                    model.is_default,
                    model.user_seq,
                    model.card_company_nm,
                )
            })
            .collect();

        Ok(user_payment_methods)
    }
}
