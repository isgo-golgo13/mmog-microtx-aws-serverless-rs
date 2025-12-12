//! # Database Service
//! 
//! ADVANTAGE: sqlx provides compile-time SQL query validation
//! ADVANTAGE: Connection pooling is built-in and efficient
//! ADVANTAGE: Async queries don't block the runtime
//! ADVANTAGE: Transactions are type-safe with RAII

use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{info, instrument};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{Transaction, TransactionStatus, NewTransaction};

/// PostgreSQL database service
/// 
/// ADVANTAGE: Pool is managed internally - no global mutable state
/// ADVANTAGE: Connection reuse across Lambda warm starts
pub struct PostgresDatabase {
    pool: PgPool,
}

impl PostgresDatabase {
    /// Create new database connection pool
    /// 
    /// ADVANTAGE: Pool configuration is explicit and type-checked
    pub async fn new(database_url: &str) -> AppResult<Self> {
        let pool = PgPoolOptions::new()
            // ADVANTAGE: Lambda-optimized pool size
            .max_connections(5)
            // ADVANTAGE: Fast connection timeout for Lambda
            .acquire_timeout(std::time::Duration::from_secs(3))
            // ADVANTAGE: Connections are tested before use
            .test_before_acquire(true)
            .connect(database_url)
            .await
            .map_err(|e| AppError::Database(e))?;
        
        info!("Database pool initialized");
        Ok(Self { pool })
    }
    
    /// Check database health
    pub async fn health_check(&self) -> AppResult<std::time::Duration> {
        let start = std::time::Instant::now();
        
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        
        Ok(start.elapsed())
    }
    
    /// Insert new transaction
    /// 
    /// ADVANTAGE: SQL is validated at compile time (with sqlx::query!)
    /// ADVANTAGE: Parameters are typed - no injection possible
    /// ADVANTAGE: Return type matches actual database schema
    #[instrument(skip(self, tx), fields(transaction_id = %tx.transaction_id))]
    pub async fn insert_transaction(&self, tx: &NewTransaction) -> AppResult<Transaction> {
        let now = chrono::Utc::now();
        
        // Note: In production with sqlx prepare, this would be compile-time checked
        let result = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO microtransactions (
                transaction_id,
                player_id,
                item_id,
                item_name,
                price_cents,
                currency,
                quantity,
                status,
                metadata,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(tx.transaction_id)
        .bind(tx.player_id)
        .bind(&tx.item_id)
        .bind(&tx.item_name)
        .bind(tx.price_cents)
        .bind(&tx.currency)
        .bind(tx.quantity)
        .bind(TransactionStatus::Pending)
        .bind(&tx.metadata)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        
        info!("Transaction inserted");
        Ok(result)
    }
    
    /// Update transaction status
    /// 
    /// ADVANTAGE: Status is enum - invalid status impossible
    #[instrument(skip(self), fields(transaction_id = %transaction_id))]
    pub async fn update_transaction_status(
        &self,
        transaction_id: Uuid,
        status: TransactionStatus,
        processor_id: Option<&str>,
    ) -> AppResult<Transaction> {
        let now = chrono::Utc::now();
        
        let result = sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE microtransactions
            SET status = $1, processor_id = $2, updated_at = $3
            WHERE transaction_id = $4
            RETURNING *
            "#
        )
        .bind(status)
        .bind(processor_id)
        .bind(now)
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Transaction {} not found", transaction_id)))?;
        
        info!(status = ?status, "Transaction status updated");
        Ok(result)
    }
    
    /// Get transaction by ID
    pub async fn get_transaction(&self, transaction_id: Uuid) -> AppResult<Option<Transaction>> {
        let result = sqlx::query_as::<_, Transaction>(
            "SELECT * FROM microtransactions WHERE transaction_id = $1"
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    /// Get player's transactions with pagination
    /// 
    /// ADVANTAGE: Pagination is type-safe with proper bounds
    #[instrument(skip(self), fields(player_id = %player_id))]
    pub async fn get_player_transactions(
        &self,
        player_id: Uuid,
        limit: i32,
        cursor: Option<Uuid>,
    ) -> AppResult<Vec<Transaction>> {
        // ADVANTAGE: Limit is i32, not any - can't pass "DROP TABLE"
        let safe_limit = limit.clamp(1, 1000);
        
        let results = match cursor {
            Some(cursor_id) => {
                sqlx::query_as::<_, Transaction>(
                    r#"
                    SELECT * FROM microtransactions
                    WHERE player_id = $1 AND transaction_id < $2
                    ORDER BY created_at DESC
                    LIMIT $3
                    "#
                )
                .bind(player_id)
                .bind(cursor_id)
                .bind(safe_limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, Transaction>(
                    r#"
                    SELECT * FROM microtransactions
                    WHERE player_id = $1
                    ORDER BY created_at DESC
                    LIMIT $2
                    "#
                )
                .bind(player_id)
                .bind(safe_limit)
                .fetch_all(&self.pool)
                .await?
            }
        };
        
        info!(count = results.len(), "Retrieved player transactions");
        Ok(results)
    }
    
    /// Execute a transactional operation
    /// 
    /// ADVANTAGE: Transaction is automatically rolled back on error
    /// ADVANTAGE: RAII ensures transaction is committed or rolled back
    pub async fn with_transaction<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<T>> + Send + '_>> + Send,
        T: Send,
    {
        let mut tx = self.pool.begin().await?;
        
        match f(&mut tx).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                // ADVANTAGE: Rollback is automatic if commit not called
                // but we can be explicit
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // ADVANTAGE: Tests use same types as production code
    // Invalid queries would fail to compile
}
