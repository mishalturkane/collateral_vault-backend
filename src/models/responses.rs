use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct CreateVaultResponse {
    pub vault_address: String,
    pub transaction: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct VaultResponse {
    pub owner: String,
    pub vault_address: String,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: DateTime<Utc>,
    pub token_mint: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub transaction: String,
    pub signature: String,
    pub estimated_fee: u64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionStatusResponse {
    pub signature: String,
    pub status: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub confirmation_status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VaultListResponse {
    pub vaults: Vec<VaultResponse>,
    pub total: usize,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub timestamp: DateTime<Utc>,
}