use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Vault {
    pub id: Uuid,
    pub owner: String,
    pub vault_address: String,
    pub token_mint: String,
    pub total_balance: i64,
    pub locked_balance: i64,
    pub available_balance: i64,
    pub total_deposited: i64,
    pub total_withdrawn: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct VaultEvent {
    pub id: Uuid,
    pub vault_owner: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TransactionLog {
    pub id: Uuid,
    pub signature: String,
    pub vault_owner: Option<String>,
    pub transaction_type: String,
    pub status: String,
    pub slot: Option<i64>,
    pub block_time: Option<i64>,
    pub fee: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuthorizedProgram {
    pub id: Uuid,
    pub program_pubkey: String,
    pub is_active: bool,
    pub added_by: Option<String>,
    pub added_at: DateTime<Utc>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<String>,
}