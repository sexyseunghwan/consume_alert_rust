use crate::common::*;

use crate::entity::{consume_prodt_detail, spent_detail, telegram_room, users, common_consume_keyword_type};
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
        spent_details: &SpentDetail
    ) -> anyhow::Result<()>;
    async fn insert_prodt_details_with_transaction(
        &self,
        spent_details: &[SpentDetail],
    ) -> anyhow::Result<()>;
    async fn exists_telegram_room_by_token_and_id(
        &self,
        room_token: &str,
        user_id: &str,
    ) -> anyhow::Result<Option<i64>>;
    async fn get_common_consume_keyword_type(
        &self,
        consume_keyword_type_id: i64
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
        spent_detail: &SpentDetail
    ) -> anyhow::Result<()> {
        
        let active_model: spent_detail::ActiveModel = spent_detail
                .convert_spent_detail_to_active_model()
                .map_err(|e| anyhow!(
                    "[MysqlQueryServiceImpl::insert_prodt_detail_with_transaction] Failed to convert to active model: {:?}",
                    e
                ))?;
        
        /* Use a transaction to insert all recored (roll back if any one fails.) */
        self.db_conn
            .insert_with_transaction(active_model)
            .await
    }

    async fn insert_prodt_details_with_transaction(
        &self,
        spent_details: &[SpentDetail],
    ) -> anyhow::Result<()> {
        /* Convert all  to ActiveModel. */
        let mut active_models: Vec<spent_detail::ActiveModel> = Vec::new();

        for spent_detail in spent_details {
            let active_model: spent_detail::ActiveModel = spent_detail
                .convert_spent_detail_to_active_model()
                .map_err(|e| anyhow!(
                    "[MysqlQueryServiceImpl::insert_prodt_detail_with_transaction] Failed to convert to active model: {:?}",
                    e
                ))?;

            active_models.push(active_model);
        }

        /* Use a transaction to insert all recored (roll back if any one fails.) */
        self.db_conn
            .insert_many_with_transaction(active_models)
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

    #[doc = "Get CommonConsumeKeywordType by consume_keyword_type_id"]
    /// # Arguments
    /// * `consume_keyword_type_id` - Primary key to search
    ///
    /// # Returns
    /// * `anyhow::Result<CommonConsumeKeywordType>` - Found model or error if not found
    async fn get_common_consume_keyword_type(
        &self,
        consume_keyword_type_id: i64
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
