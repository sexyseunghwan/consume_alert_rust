use crate::common::*;
use crate::entity::spent_detail;
use sea_orm::{ActiveModelBehavior, IntoActiveModel};

#[async_trait]
pub trait MysqlRepository {
    /// Generic insert function that can insert any ActiveModel.
    ///
    /// # Arguments
    /// * `active_model` - The ActiveModel instance to insert
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if insert succeeds
    async fn insert<A>(&self, active_model: A) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>;

    /// Generic bulk insert function.
    ///
    /// # Arguments
    /// * `active_models` - Vector of ActiveModel instances to insert
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if all inserts succeed
    async fn insert_many<A>(&self, active_models: Vec<A>) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>;

    /// Insert a single ActiveModel within a transaction.
    ///
    /// # Arguments
    /// * `active_model` - The ActiveModel instance to insert
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if insert succeeds
    async fn insert_with_transaction<A>(&self, active_model: A) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>;

    /// Bulk insert with transaction - rolls back entirely if any insert fails.
    ///
    /// # Arguments
    /// * `active_models` - Vector of ActiveModel instances to insert
    ///
    /// # Returns
    /// * `Result<(), anyhow::Error>` - Ok if all inserts succeed, rolls back on failure
    async fn insert_many_with_transaction<A>(&self, active_models: Vec<A>) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>;

    /// Inserts multiple [`spent_detail::ActiveModel`] records one by one within a single
    /// transaction and returns the auto-incremented `spent_idx` assigned by the database
    /// for each record.
    ///
    /// # Why one-by-one instead of bulk insert
    ///
    /// SeaORM's `insert_many` only exposes the last inserted ID, making it impossible
    /// to recover individual IDs for a batch.  Inserting sequentially inside one
    /// transaction lets us capture each `last_insert_id` immediately after its row is
    /// written, while still guaranteeing atomicity.
    ///
    /// # Ordering guarantee
    ///
    /// The returned `Vec<i64>` is in the **same order** as `active_models`.
    /// Each `ids[i]` is the `spent_idx` assigned to `active_models[i]`.
    /// This holds because the loop is strictly sequential — no parallelism is involved.
    ///
    /// # Arguments
    ///
    /// * `active_models` - Records to insert, in the order they should be tracked.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<i64>)` - Auto-incremented `spent_idx` values, one per input record,
    ///   in insertion order.
    /// * `Err` - The transaction is rolled back and the error is propagated.
    async fn insert_spent_details_with_transaction(
        &self,
        active_models: Vec<spent_detail::ActiveModel>,
    ) -> anyhow::Result<Vec<i64>>;

    /// Returns a reference to the DatabaseConnection.
    ///
    /// # Returns
    /// * `&DatabaseConnection` - Reference to the database connection
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
    #[doc = "Insert a single ActiveModel into the database"]
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

    #[doc = "Insert multiple ActiveModels into the database in a single query"]
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

    #[doc = "Insert a single ActiveModel within a database transaction"]
    async fn insert_with_transaction<A>(&self, active_model: A) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    {
        // Begin transaction.
        let txn: DatabaseTransaction = self.db_conn.begin().await.map_err(|e| {
            anyhow!(
                "[MysqlRepositoryImpl::insert_with_transaction] Failed to begin transaction: {:?}",
                e
            )
        })?;

        // Insert data in a single query for better performance.
        A::Entity::insert(active_model)
            .exec(&txn)
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlRepositoryImpl::insert_with_transaction] Failed to bulk insert: {:?}",
                    e
                )
            })?;

        // Commit if all inserts succeed.
        txn.commit().await.map_err(|e| {
            anyhow!(
                "[MysqlRepositoryImpl::insert_with_transaction] Failed to commit transaction: {:?}",
                e
            )
        })?;

        Ok(())
    }

    #[doc = "Insert multiple ActiveModels within a database transaction"]
    /// All inserts are executed in a single query for better performance.
    /// If any insert fails, the entire transaction is rolled back.
    async fn insert_many_with_transaction<A>(&self, active_models: Vec<A>) -> anyhow::Result<()>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <A::Entity as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    {
        if active_models.is_empty() {
            return Ok(());
        }

        // Begin transaction.
        let txn: DatabaseTransaction = self.db_conn
            .begin()
            .await
            .map_err(|e| anyhow!("[MysqlRepositoryImpl::insert_many_with_transaction] Failed to begin transaction: {:?}", e))?;

        // Bulk insert - executes all inserts in a single query for better performance.
        A::Entity::insert_many(active_models)
            .exec(&txn)
            .await
            .map_err(|e| {
                anyhow!(
                    "[MysqlRepositoryImpl::insert_many_with_transaction] Failed to bulk insert: {:?}",
                    e
                )
            })?;

        // Commit if all inserts succeed.
        txn.commit()
            .await
            .map_err(|e| anyhow!("[MysqlRepositoryImpl::insert_many_with_transaction] Failed to commit transaction: {:?}", e))?;

        Ok(())
    }

    async fn insert_spent_details_with_transaction(
        &self,
        active_models: Vec<spent_detail::ActiveModel>,
    ) -> anyhow::Result<Vec<i64>> {
        if active_models.is_empty() {
            return Ok(vec![]);
        }

        // Open a single transaction that wraps all inserts.
        // If any insert fails the transaction is rolled back automatically
        // when `txn` is dropped without being committed.
        let txn: DatabaseTransaction = self.db_conn
            .begin()
            .await
            .map_err(|e| anyhow!(
                "[MysqlRepositoryImpl::insert_spent_details_with_transaction] Failed to begin transaction: {:?}",
                e
            ))?;

        // `inserted_ids[i]` will hold the `spent_idx` assigned to `active_models[i]`.
        // Pre-allocating avoids reallocations inside the loop.
        let mut inserted_ids: Vec<i64> = Vec::with_capacity(active_models.len());

        for (position, active_model) in active_models.into_iter().enumerate() {
            let insert_result: InsertResult<spent_detail::ActiveModel> =
                spent_detail::Entity::insert(active_model)
                    .exec(&txn)
                    .await
                    .map_err(|e| {
                        anyhow!(
                            "[MysqlRepositoryImpl::insert_spent_details_with_transaction] \
                         Failed to insert record at position {}: {:?}",
                            position,
                            e
                        )
                    })?;

            // `last_insert_id` is the AUTO_INCREMENT value the DB assigned to this row.
            // Pushing immediately after the insert preserves insertion order.
            inserted_ids.push(insert_result.last_insert_id);
        }

        txn.commit()
            .await
            .map_err(|e| anyhow!(
                "[MysqlRepositoryImpl::insert_spent_details_with_transaction] Failed to commit transaction: {:?}",
                e
            ))?;

        Ok(inserted_ids)
    }

    #[doc = "Get a reference to the underlying database connection"]
    fn get_connection(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}
