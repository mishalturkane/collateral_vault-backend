use sqlx::{PgPool, postgres::PgPoolOptions};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await
            .expect("Failed to connect to database");
        
        Self { pool }
    }
    
    pub async fn create_vault(
        &self,
        owner: &str,
        vault_address: &str,
        token_account: &str,
        bump: u8,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO vaults (owner, vault_address, token_account, bump, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
            owner,
            vault_address,
            token_account,
            bump as i16,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_vault_by_owner(&self, owner: &str) -> Result<VaultRecord, sqlx::Error> {
        let record = sqlx::query_as!(
            VaultRecord,
            r#"
            SELECT 
                id, owner, vault_address, token_account,
                total_balance, locked_balance, available_balance,
                total_deposited, total_withdrawn, created_at, updated_at, bump, status
            FROM vaults WHERE owner = $1
            "#,
            owner
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(record)
    }
    
    pub async fn update_vault_balance(
        &self,
        owner: &str,
        total_delta: i64,
        locked_delta: i64,
        available_delta: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE vaults 
            SET 
                total_balance = total_balance + $2,
                locked_balance = locked_balance + $3,
                available_balance = available_balance + $4,
                updated_at = NOW()
            WHERE owner = $1
            "#,
            owner,
            total_delta,
            locked_delta,
            available_delta,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn record_transaction(
        &self,
        owner: &str,
        tx_type: &str,
        amount: i64,
        signature: &str,
        from_vault: Option<&str>,
        to_vault: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let metadata = serde_json::json!({
            "from_vault": from_vault,
            "to_vault": to_vault,
        });
        
        sqlx::query!(
            r#"
            INSERT INTO transactions (vault_owner, tx_type, amount, signature, metadata, timestamp)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
            owner,
            tx_type,
            amount,
            signature,
            metadata,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_transaction_history(
        &self,
        owner: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TransactionRecord>, sqlx::Error> {
        let records = sqlx::query_as!(
            TransactionRecord,
            r#"
            SELECT 
                id, vault_owner, tx_type, amount, signature, metadata, timestamp
            FROM transactions 
            WHERE vault_owner = $1
            ORDER BY timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
            owner,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(records)
    }
    
    pub async fn get_total_value_locked(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT COALESCE(SUM(total_balance), 0) as tvl FROM vaults"#
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result.tvl.unwrap_or(0))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultRecord {
    pub id: i32,
    pub owner: String,
    pub vault_address: String,
    pub token_account: String,
    pub total_balance: i64,
    pub locked_balance: i64,
    pub available_balance: i64,
    pub total_deposited: i64,
    pub total_withdrawn: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bump: i16,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub id: i32,
    pub vault_owner: String,
    pub tx_type: String,
    pub amount: i64,
    pub signature: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}