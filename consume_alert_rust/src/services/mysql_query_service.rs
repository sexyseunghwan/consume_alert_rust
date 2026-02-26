use crate::common::*;

use crate::entity::{
    common_consume_keyword_type, consume_prodt_detail, spent_detail, telegram_room, users,
};
use crate::models::{common_consume_keyword_type::*, consume_prodt_info::*, spent_detail::*};
use crate::repository::mysql_repository::*;

#[async_trait]
pub trait MysqlQueryService {
    async fn insert_consume_prodt_detail(
        &self,
        consume_info: &ConsumeProdtInfo,
    ) -> anyhow::Result<()>;
    async fn insert_prodt_detail_with_transaction(
        &self,
        spent_details: &SpentDetail,
    ) -> anyhow::Result<()>;
    /// Converts each [`SpentDetail`] to a SeaORM `ActiveModel`, then delegates to
    /// the repository to insert them all within a single database transaction.
    ///
    /// # Ordering guarantee
    ///
    /// The returned `Vec<i64>` is in the **same order** as `spent_details`.
    /// `spent_idxs[i]` is the auto-incremented primary key assigned by the database
    /// to `spent_details[i]`.
    ///
    /// # Arguments
    ///
    /// * `spent_details` - Slice of domain model records to persist, in the order
    ///   they should be tracked.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<i64>)` - Assigned `spent_idx` values, one per input record,
    ///   in the same order as `spent_details`.
    /// * `Err` - Conversion or database failure; the transaction is rolled back.
    async fn insert_prodt_details_with_transaction(
        &self,
        spent_details: &[SpentDetail],
    ) -> anyhow::Result<Vec<i64>>;
    async fn exists_telegram_room_by_token_and_id(
        &self,
        room_token: &str,
        user_id: &str,
    ) -> anyhow::Result<Option<i64>>;
    async fn get_telegram_room_seq_by_token_and_userseq(
        &self,
        room_token: &str,
        user_seq: i64,
    ) -> anyhow::Result<Option<i64>>;
    async fn get_common_consume_keyword_type(
        &self,
        consume_keyword_type_id: i64,
    ) -> anyhow::Result<CommonConsumeKeywordType>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct MysqlQueryServiceImpl<R: MysqlRepository> {
    db_conn: R,
}

impl<R: MysqlRepository> MysqlQueryServiceImpl<R> {}

#[async_trait]
impl<R: MysqlRepository + Send + Sync> MysqlQueryService for MysqlQueryServiceImpl<R> {
    #[doc = ""]
    async fn insert_consume_prodt_detail(
        &self,
        consume_info: &ConsumeProdtInfo,
    ) -> anyhow::Result<()> {
        /* ConsumeProdtInfo -> ActiveModel */
        let active_model: consume_prodt_detail::ActiveModel = consume_info
            .convert_consume_info_to_active_model()
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::insert_consume_prodt_detail] active_model: {:?}",
                    e
                )
            })?;

        self.db_conn.insert(active_model).await
    }

    async fn insert_prodt_detail_with_transaction(
        &self,
        spent_detail: &SpentDetail,
    ) -> anyhow::Result<()> {
        let active_model: spent_detail::ActiveModel = spent_detail
                .convert_spent_detail_to_active_model()
                .map_err(|e| anyhow!(
                    "[MysqlQueryServiceImpl::insert_prodt_detail_with_transaction] Failed to convert to active model: {:?}",
                    e
                ))?;

        /* Use a transaction to insert all recored (roll back if any one fails.) */
        self.db_conn.insert_with_transaction(active_model).await
    }

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

    #[doc = "Get CommonConsumeKeywordType by consume_keyword_type_id"]
    /// # Arguments
    /// * `consume_keyword_type_id` - Primary key to search
    ///
    /// # Returns
    /// * `anyhow::Result<CommonConsumeKeywordType>` - Found model or error if not found
    async fn get_common_consume_keyword_type(
        &self,
        consume_keyword_type_id: i64,
    ) -> anyhow::Result<CommonConsumeKeywordType> {
        let result: Option<common_consume_keyword_type::Model> = common_consume_keyword_type::Entity::find()
            .filter(common_consume_keyword_type::Column::ConsumeKeywordTypeId.eq(consume_keyword_type_id))
            .one(self.db_conn.get_connection())
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlQueryServiceImpl::get_common_consume_keyword_type] Failed to query: {:?}",
                    e
                )
            })?;

        let entity: common_consume_keyword_type::Model = result.ok_or_else(|| {
            anyhow!(
                "[MysqlQueryServiceImpl::get_common_consume_keyword_type] Not found: consume_keyword_type_id = {}",
                consume_keyword_type_id
            )
        })?;

        Ok(CommonConsumeKeywordType::new(
            entity.consume_keyword_type_id,
            entity.consume_keyword_type,
        ))
    }
}
