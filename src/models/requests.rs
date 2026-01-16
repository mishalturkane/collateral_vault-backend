use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateVaultRequest {
    #[validate(length(min = 32, max = 44))]
    pub owner: String,
    
    #[validate(length(min = 32, max = 44))]
    pub token_mint: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DepositRequest {
    #[validate(range(min = 1))]
    pub amount: u64,
    
    #[validate(length(min = 32, max = 44))]
    pub user_token_account: String,
    
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct WithdrawRequest {
    #[validate(range(min = 1))]
    pub amount: u64,
    
    #[validate(length(min = 32, max = 44))]
    pub user_token_account: String,
    
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LockRequest {
    #[validate(range(min = 1))]
    pub amount: u64,
    
    #[validate(length(min = 32, max = 44))]
    pub caller_program: String,
    
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UnlockRequest {
    #[validate(range(min = 1))]
    pub amount: u64,
    
    #[validate(length(min = 32, max = 44))]
    pub caller_program: String,
    
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct TransferRequest {
    #[validate(length(min = 32, max = 44))]
    pub to_owner: String,
    
    #[validate(range(min = 1))]
    pub amount: u64,
    
    #[validate(length(min = 32, max = 44))]
    pub caller_program: String,
    
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct InitializeAuthorityRequest {
    #[validate(length(max = 10))]
    pub authorized_programs: Vec<String>,
    
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddAuthorizedProgramRequest {
    #[validate(length(min = 32, max = 44))]
    pub program: String,
    
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct BuildTransactionRequest {
    pub parameters: serde_json::Value,
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SubmitTransactionRequest {
    #[validate(length(min = 88, max = 176))]
    pub signed_transaction: String,
}