use crate::common::*;

use crate::entity::consume_prodt_detail;
use crate::models::consume_prodt_info::*;
use crate::repository::mysql_repository::*;

#[async_trait]
pub trait MysqlQueryService {
    async fn insert_consume_prodt_detail(
        &self,
        consume_info: &ConsumeProdtInfo,
    ) -> anyhow::Result<()>;

    /// Transaction을 사용하여 여러 consume_prodt_detail을 insert (하나라도 실패하면 전체 rollback)
    async fn insert_consume_prodt_details_with_transaction(
        &self,
        consume_infos: &Vec<ConsumeProdtInfo>,
    ) -> anyhow::Result<()>;
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

    #[doc = ""]
    async fn insert_consume_prodt_details_with_transaction(
        &self,
        consume_infos: &Vec<ConsumeProdtInfo>,
    ) -> anyhow::Result<()> {
        /* Convert all ConsumeProductInfo to ActiveModel. */
        let mut active_models: Vec<consume_prodt_detail::ActiveModel> = Vec::new();

        for consume_info in consume_infos {
            let active_model: consume_prodt_detail::ActiveModel = consume_info
                .convert_consume_info_to_active_model()
                .map_err(|e| anyhow!(
                    "[MysqlQueryServiceImpl::insert_consume_prodt_details_with_transaction] Failed to convert to active model: {:?}",
                    e
                ))?;

            active_models.push(active_model);
        }

        /* Use a transaction to insert all recored (roll back if any one fails.) */
        self.db_conn
            .insert_many_with_transaction(active_models)
            .await
    }
}
