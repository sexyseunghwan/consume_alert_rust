use crate::common::*;

use crate::entity::{spent_detail, telegram_room, users};
use crate::models::spent_detail::*;
use crate::repository::mysql_repository::*;

use crate::service_traits::mysql_query_service::*;

#[derive(Debug, Getters, Clone, new)]
pub struct MysqlQueryServiceImpl<R: MysqlRepository> {
    db_conn: R,
}

#[async_trait]
impl<R: MysqlRepository + Send + Sync> MysqlQueryService for MysqlQueryServiceImpl<R> {
    async fn insert_prodt_detail_with_transaction(
        &self,
        spent_detail: &SpentDetail,
    ) -> anyhow::Result<i64> {
        let active_model = spent_detail
            .convert_spent_detail_to_active_model()
            .context("[insert_prodt_detail_with_transaction] Failed to convert to ActiveModel")?;

        self.db_conn
            .insert_spent_detail_with_transaction(active_model)
            .await
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
}
