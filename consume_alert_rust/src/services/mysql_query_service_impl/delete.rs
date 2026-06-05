use crate::repository::mysql_repository::*;

use super::MysqlQueryServiceImpl;

impl<R: MysqlRepository + Send + Sync> MysqlQueryServiceImpl<R> {
    pub async fn delete_spent_detail_with_transaction(&self, spent_idx: i64) -> anyhow::Result<()> {
        self.db_conn
            .delete_spent_detail_with_transaction(spent_idx)
            .await
    }
}
