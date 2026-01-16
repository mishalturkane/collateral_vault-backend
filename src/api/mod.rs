use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use crate::vault_manager::VaultManager;

#[derive(Debug, Deserialize)]
pub struct InitializeRequest {
    pub user_pubkey: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub user_pubkey: String,
    pub amount: u64,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub fn routes(vault_manager: Arc<VaultManager>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let api_base = warp::path("api");
    
    // Initialize vault endpoint
    let initialize = warp::path!("vault" / "initialize")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_vault_manager(vault_manager.clone()))
        .and_then(handle_initialize);
    
    // Deposit endpoint
    let deposit = warp::path!("vault" / "deposit")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_vault_manager(vault_manager.clone()))
        .and_then(handle_deposit);
    
    // Withdrawal endpoint
    let withdraw = warp::path!("vault" / "withdraw")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_vault_manager(vault_manager.clone()))
        .and_then(handle_withdraw);
    
    // Get balance endpoint
    let get_balance = warp::path!("vault" / "balance" / String)
        .and(warp::get())
        .and(with_vault_manager(vault_manager.clone()))
        .and_then(handle_get_balance);
    
    // Get transaction history
    let get_transactions = warp::path!("vault" / "transactions" / String)
        .and(warp::get())
        .and(with_vault_manager(vault_manager.clone()))
        .and_then(handle_get_transactions);
    
    // Get TVL
    let get_tvl = warp::path!("vault" / "tvl")
        .and(warp::get())
        .and(with_vault_manager(vault_manager))
        .and_then(handle_get_tvl);
    
    api_base.and(
        initialize
        .or(deposit)
        .or(withdraw)
        .or(get_balance)
        .or(get_transactions)
        .or(get_tvl)
    )
}

fn with_vault_manager(
    vault_manager: Arc<VaultManager>,
) -> impl Filter<Extract = (Arc<VaultManager>,), Error = Infallible> + Clone {
    warp::any().map(move || vault_manager.clone())
}

async fn handle_initialize(
    req: InitializeRequest,
    vault_manager: Arc<VaultManager>,
) -> Result<impl Reply, Rejection> {
    match vault_manager.initialize_user_vault(&req.user_pubkey).await {
        Ok(result) => Ok(warp::reply::json(&ApiResponse {
            success: true,
            data: Some(result),
            error: None,
        })),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<String> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

async fn handle_deposit(
    req: DepositRequest,
    vault_manager: Arc<VaultManager>,
) -> Result<impl Reply, Rejection> {
    // In real implementation, you'd get the keypair from secure storage
    // This is simplified
    let user_keypair = Keypair::new();
    
    match vault_manager.deposit_collateral(&user_keypair, req.amount, &req.signature).await {
        Ok(result) => Ok(warp::reply::json(&ApiResponse {
            success: true,
            data: Some(result),
            error: None,
        })),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<String> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

async fn handle_get_balance(
    user_pubkey: String,
    vault_manager: Arc<VaultManager>,
) -> Result<impl Reply, Rejection> {
    match vault_manager.get_vault_balance(&user_pubkey).await {
        Ok(balance) => Ok(warp::reply::json(&ApiResponse {
            success: true,
            data: Some(balance),
            error: None,
        })),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<VaultBalance> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

async fn handle_get_tvl(
    vault_manager: Arc<VaultManager>,
) -> Result<impl Reply, Rejection> {
    // This would query the database for total value locked
    // Simplified for now
    Ok(warp::reply::json(&ApiResponse {
        success: true,
        data: Some(1000000), // Example TVL
        error: None,
    }))
}