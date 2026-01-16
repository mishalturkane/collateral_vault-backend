use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
    response::{Response, IntoResponse, sse::Event},
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    requests::*,
    responses::*,
    database::Vault as DbVault,
};
use crate::database::DatabasePool;
use crate::services::vault::VaultService;
use crate::utils::error::{ApiError, ResultExt};

type ApiResult<T> = Result<Json<T>, ApiError>;

pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn create_vault(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Json(request): Json<CreateVaultRequest>,
) -> ApiResult<CreateVaultResponse> {
    request.validate()?;
    
    let result = vault_service.initialize_vault(
        &request.owner,
        &request.token_mint,
    ).await?;
    
    Ok(Json(CreateVaultResponse {
        vault_address: result.vault_address,
        transaction: result.transaction,
        message: "Vault initialized successfully".to_string(),
    }))
}

pub async fn get_vault(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(owner): Path<String>,
) -> ApiResult<VaultResponse> {
    let vault_info = vault_service.get_vault_info(&owner).await?;
    
    Ok(Json(VaultResponse {
        owner: vault_info.owner,
        vault_address: vault_info.vault_address,
        total_balance: vault_info.total_balance,
        locked_balance: vault_info.locked_balance,
        available_balance: vault_info.available_balance,
        total_deposited: vault_info.total_deposited,
        total_withdrawn: vault_info.total_withdrawn,
        created_at: vault_info.created_at,
        token_mint: vault_info.token_mint,
    }))
}

pub async fn deposit(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(owner): Path<String>,
    Json(request): Json<DepositRequest>,
) -> ApiResult<TransactionResponse> {
    request.validate()?;
    
    let result = vault_service.deposit_collateral(
        &owner,
        request.amount,
        &request.user_token_account,
        request.priority_fee,
    ).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Deposit transaction created".to_string(),
    }))
}

pub async fn withdraw(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(owner): Path<String>,
    Json(request): Json<WithdrawRequest>,
) -> ApiResult<TransactionResponse> {
    request.validate()?;
    
    let result = vault_service.withdraw_collateral(
        &owner,
        request.amount,
        &request.user_token_account,
        request.priority_fee,
    ).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Withdrawal transaction created".to_string(),
    }))
}

pub async fn lock_collateral(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(owner): Path<String>,
    Json(request): Json<LockRequest>,
) -> ApiResult<TransactionResponse> {
    request.validate()?;
    
    let result = vault_service.lock_collateral(
        &owner,
        request.amount,
        &request.caller_program,
        request.priority_fee,
    ).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Collateral locked".to_string(),
    }))
}

pub async fn unlock_collateral(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(owner): Path<String>,
    Json(request): Json<UnlockRequest>,
) -> ApiResult<TransactionResponse> {
    request.validate()?;
    
    let result = vault_service.unlock_collateral(
        &owner,
        request.amount,
        &request.caller_program,
        request.priority_fee,
    ).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Collateral unlocked".to_string(),
    }))
}

pub async fn transfer_collateral(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(from_owner): Path<String>,
    Json(request): Json<TransferRequest>,
) -> ApiResult<TransactionResponse> {
    request.validate()?;
    
    let result = vault_service.transfer_collateral(
        &from_owner,
        &request.to_owner,
        request.amount,
        &request.caller_program,
        request.priority_fee,
    ).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Collateral transferred".to_string(),
    }))
}

pub async fn close_vault(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(owner): Path<String>,
) -> ApiResult<TransactionResponse> {
    let result = vault_service.close_vault(&owner).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Vault closed successfully".to_string(),
    }))
}

pub async fn initialize_authority(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Json(request): Json<InitializeAuthorityRequest>,
) -> ApiResult<TransactionResponse> {
    request.validate()?;
    
    let result = vault_service.initialize_authority(
        &request.authorized_programs,
        request.priority_fee,
    ).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Authority initialized".to_string(),
    }))
}

pub async fn add_authorized_program(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Json(request): Json<AddAuthorizedProgramRequest>,
) -> ApiResult<TransactionResponse> {
    request.validate()?;
    
    let result = vault_service.add_authorized_program(
        &request.program,
        request.priority_fee,
    ).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Program authorized".to_string(),
    }))
}

pub async fn remove_authorized_program(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(program): Path<String>,
) -> ApiResult<TransactionResponse> {
    let result = vault_service.remove_authorized_program(&program).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Program deauthorized".to_string(),
    }))
}

pub async fn build_transaction(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(tx_type): Path<String>,
    Json(request): Json<BuildTransactionRequest>,
) -> ApiResult<TransactionResponse> {
    let result = vault_service.build_transaction(
        &tx_type,
        &request.parameters,
        request.priority_fee,
    ).await?;
    
    Ok(Json(TransactionResponse {
        transaction: result.transaction,
        signature: result.signature,
        estimated_fee: result.estimated_fee,
        message: "Transaction built".to_string(),
    }))
}

pub async fn submit_transaction(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Json(request): Json<SubmitTransactionRequest>,
) -> ApiResult<TransactionStatusResponse> {
    let result = vault_service.submit_transaction(
        &request.signed_transaction,
    ).await?;
    
    Ok(Json(TransactionStatusResponse {
        signature: result.signature,
        status: result.status,
        slot: result.slot,
        block_time: result.block_time,
        confirmation_status: result.confirmation_status,
        error: result.error,
    }))
}

pub async fn get_transaction_status(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
    Path(signature): Path<String>,
) -> ApiResult<TransactionStatusResponse> {
    let result = vault_service.get_transaction_status(&signature).await?;
    
    Ok(Json(TransactionStatusResponse {
        signature: result.signature,
        status: result.status,
        slot: result.slot,
        block_time: result.block_time,
        confirmation_status: result.confirmation_status,
        error: result.error,
    }))
}

pub async fn stream_events(
    State((pool, vault_service)): State<(DatabasePool, VaultService)>,
) -> impl IntoResponse {
    let stream = vault_service.stream_events().await;
    
    axum::response::Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
}

pub async fn auth_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<impl IntoResponse, ApiError> {
    // Extract and validate API key or JWT token
    let api_key = request.headers()
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;
    
    // Validate API key (simplified)
    if api_key != "your-secure-api-key" {
        return Err(ApiError::Unauthorized);
    }
    
    Ok(next.run(request).await)
}