use crate::common::*;

use crate::entity::{common_consume_keyword_type, spent_detail, telegram_room, user_payment_methods, users};
use crate::models::spent_detail::*;
use crate::models::spent_detail_with_info::*;
use crate::models::user_payment_methods::*;
use crate::repository::mysql_repository::*;

use crate::service_traits::mysql_query_service::*;

#[derive(Debug, Getters, Clone, new)]
pub struct MysqlQueryServiceImpl<R: MysqlRepository> {
    db_conn: R,
}

impl<R: MysqlRepository + Send + Sync> MysqlQueryServiceImpl<R> {

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

    async fn delete_spent_detail_with_transaction(&self, spent_idx: i64) -> anyhow::Result<()> {
        self.db_conn
            .delete_spent_detail_with_transaction(spent_idx)
            .await
    }

    async fn get_user_payment_methods(
        &self,
        user_seq: i64,
        is_default: bool
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
                    model.updated_at.map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
                    model.created_by,
                    model.updated_by,
                    model.is_default,
                    model.user_seq,
                    model.card_company_nm
                )
            })
            .collect();
        
        Ok(user_payment_methods)
    }
}