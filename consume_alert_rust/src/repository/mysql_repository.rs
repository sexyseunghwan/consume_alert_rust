use crate::common::*;
use sea_orm::{ActiveModelBehavior, IntoActiveModel};

#[async_trait]
pub trait MysqlRepository {
    /// 제네릭 insert 함수 - 어떤 ActiveModel이든 insert 가능
    async fn insert<A>(&self, active_model: A) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>;

    /// 제네릭 bulk insert 함수
    async fn insert_many<A>(&self, active_models: Vec<A>) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>;

    /// Transaction을 사용한 bulk insert - 하나라도 실패하면 전체 rollback
    async fn insert_many_with_transaction<A>(&self, active_models: Vec<A>) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>;

    /// DatabaseConnection 참조 반환
    fn get_connection(&self) -> &DatabaseConnection;
}

pub struct MysqlRepositoryImpl {
    db_conn: DatabaseConnection,
}

impl MysqlRepositoryImpl {
    pub async fn new() -> anyhow::Result<Self> {
        let db_url: String = env::var("DATABASE_URL")
            .expect("[MysqlRepositoryImpl::new] DATABASE_URL must be set in .env");

        let db_conn: DatabaseConnection = Database::connect(db_url)
            .await
            .expect("[MysqlRepositoryImpl::new] Database connection failed");

        Ok(Self { db_conn })
    }
}

#[async_trait]
impl MysqlRepository for MysqlRepositoryImpl {
    
    #[doc = ""]
    async fn insert<A>(&self, active_model: A) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    {
        active_model
            .insert(&self.db_conn)
            .await
            .map_err(|e| anyhow!("[MysqlRepositoryImpl::insert] Failed to insert: {:?}", e))?;

        Ok(())
    }

    #[doc = ""]
    async fn insert_many<A>(&self, active_models: Vec<A>) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    {
        if active_models.is_empty() {
            return Ok(());
        }

        A::Entity::insert_many(active_models)
            .exec(&self.db_conn)
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlRepositoryImpl::insert_many] Failed to bulk insert: {:?}",
                    e
                )
            })?;

        Ok(())
    }

    #[doc = ""]
    async fn insert_many_with_transaction<A>(&self, active_models: Vec<A>) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    {
        if active_models.is_empty() {
            return Ok(());
        }
        
        // Start Transaction
        let txn: DatabaseTransaction = self.db_conn
            .begin()
            .await
            .map_err(|e| anyhow!("[MysqlRepositoryImpl::insert_many_with_transaction] Failed to begin transaction: {:?}", e))?;

        // Bulk insert - 한 번의 쿼리로 모든 데이터 insert (성능 향상)
        A::Entity::insert_many(active_models)
            .exec(&txn)
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlRepositoryImpl::insert_many_with_transaction] Failed to bulk insert: {:?}",
                    e
                )
            })?;

        // 모두 성공하면 commit
        txn.commit()
            .await
            .map_err(|e| anyhow!("[MysqlRepositoryImpl::insert_many_with_transaction] Failed to commit transaction: {:?}", e))?;

        Ok(())
    }

    fn get_connection(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}
