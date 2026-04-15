use crate::common::*;
use crate::entity::spent_detail;

#[async_trait]
pub trait MysqlRepository {
    /// Inserts a single [`spent_detail::ActiveModel`] within a transaction and returns
    /// the auto-incremented `spent_idx` assigned by the database.
    ///
    /// # Arguments
    ///
    /// * `active_model` - The record to insert.
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` - The `spent_idx` assigned to the inserted row.
    /// * `Err`     - The transaction is rolled back and the error is propagated.
    async fn insert_spent_detail_with_transaction(
        &self,
        active_model: spent_detail::ActiveModel,
    ) -> anyhow::Result<i64>;

    #[allow(dead_code)]
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

    /// Deletes a single [`spent_detail`] row identified by `spent_idx` within a transaction.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Row deleted and transaction committed.
    /// * `Err`    - The transaction is rolled back and the error is propagated.
    async fn delete_spent_detail_with_transaction(&self, spent_idx: i64) -> anyhow::Result<()>;

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
    /// Creates a new `MysqlRepositoryImpl` by reading the database URL from environment variables and establishing a connection.
    ///
    /// # Returns
    ///
    /// Returns `Ok(MysqlRepositoryImpl)` on successful database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the `DATABASE_URL` environment variable is not set or the connection fails.
    pub async fn new() -> anyhow::Result<Self> {
        let db_url: String = env::var("DATABASE_URL")
            .inspect_err(|e| {
                error!("[MysqlRepositoryImpl::new] DATABASE_URL must be set: {:#}", e);
            })?;
            //.expect("[MysqlRepositoryImpl::new] DATABASE_URL must be set in .env");

        let db_conn: DatabaseConnection = Database::connect(db_url)
            .await
            .inspect_err(|e| {
                error!("[MysqlRepositoryImpl::new] Database connection failed.: {:#}", e);
            })?;

        Ok(Self { db_conn })
    }
}

#[async_trait]
impl MysqlRepository for MysqlRepositoryImpl {
    /// Inserts a single `spent_detail` record within a transaction and returns the generated primary key.
    ///
    /// # Arguments
    ///
    /// * `active_model` - The SeaORM active model representing the record to insert
    ///
    /// # Returns
    ///
    /// Returns `Ok(i64)` with the auto-incremented `spent_idx` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if beginning, executing, or committing the transaction fails.
    async fn insert_spent_detail_with_transaction(
        &self,
        active_model: spent_detail::ActiveModel,
    ) -> anyhow::Result<i64> {
        let txn: DatabaseTransaction = self
            .db_conn
            .begin()
            .await
            .context("[MysqlRepositoryImpl::insert_spent_detail_with_transaction] Failed to begin transaction")?;

        let insert_result: InsertResult<spent_detail::ActiveModel> = spent_detail::Entity::insert(
            active_model,
        )
        .exec(&txn)
        .await
        .context(
            "[MysqlRepositoryImpl::insert_spent_detail_with_transaction] Failed to insert record",
        )?;

        txn.commit()
            .await
            .context("[MysqlRepositoryImpl::insert_spent_detail_with_transaction] Failed to commit transaction")?;

        Ok(insert_result.last_insert_id)
    }

    /// Inserts multiple `spent_detail` records sequentially within a single transaction, returning all generated primary keys.
    ///
    /// # Arguments
    ///
    /// * `active_models` - Vector of SeaORM active models to insert in order
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<i64>)` with `spent_idx` values in the same order as `active_models`.
    ///
    /// # Errors
    ///
    /// Returns an error if any insert or the transaction commit fails; the entire transaction is rolled back.
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

    /// Deletes a `spent_detail` row identified by `spent_idx` within a transaction.
    ///
    /// # Arguments
    ///
    /// * `spent_idx` - The primary key of the record to delete
    ///
    /// # Errors
    ///
    /// Returns an error if beginning, executing the delete, or committing the transaction fails.
    async fn delete_spent_detail_with_transaction(&self, spent_idx: i64) -> anyhow::Result<()> {
        let txn: DatabaseTransaction = self
            .db_conn
            .begin()
            .await
            .map_err(|e| anyhow!(
                "[MysqlRepositoryImpl::delete_spent_detail_with_transaction] Failed to begin transaction: {:?}",
                e
            ))?;

        spent_detail::Entity::delete_by_id(spent_idx)
            .exec(&txn)
            .await
            .map_err(|e| anyhow!(
                "[MysqlRepositoryImpl::delete_spent_detail_with_transaction] Failed to delete record: {:?}",
                e
            ))?;

        txn.commit()
            .await
            .map_err(|e| anyhow!(
                "[MysqlRepositoryImpl::delete_spent_detail_with_transaction] Failed to commit transaction: {:?}",
                e
            ))?;

        Ok(())
    }

    #[doc = "Get a reference to the underlying database connection"]
    fn get_connection(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}
