use std::sync::Arc;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcSendTransactionConfig, RpcTransactionConfig},
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Signature,
    transaction::Transaction,
};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RpcService {
    rpc_client: Arc<Mutex<RpcClient>>,
    commitment: CommitmentConfig,
}

impl RpcService {
    pub fn new(rpc_url: &str) -> Result<Self> {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        
        Ok(Self {
            rpc_client: Arc::new(Mutex::new(rpc_client)),
            commitment: CommitmentConfig::confirmed(),
        })
    }
    
    pub async fn send_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature> {
        let client = self.rpc_client.lock().await;
        
        let config = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(self.commitment.commitment),
            encoding: None,
            max_retries: None,
            min_context_slot: None,
        };
        
        let signature = client.send_transaction_with_config(transaction, config)
            .context("Failed to send transaction")?;
        
        Ok(signature)
    }
    
    pub async fn confirm_transaction(
        &self,
        signature: &Signature,
    ) -> Result<()> {
        let client = self.rpc_client.lock().await;
        
        client.confirm_transaction_with_commitment(
            signature,
            &self.commitment,
        )?;
        
        Ok(())
    }
    
    pub async fn get_fee_for_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<u64> {
        let client = self.rpc_client.lock().await;
        
        let recent_blockhash = client.get_latest_blockhash()?;
        let fee_calculator = client.get_fee_calculator_for_blockhash(&recent_blockhash)?;
        
        let fee = fee_calculator
            .map(|calc| calc.lamports_per_signature * transaction.signatures.len() as u64)
            .unwrap_or(5000); // Default fee
        
        Ok(fee)
    }
    
    pub async fn get_transaction_status(
        &self,
        signature: &Signature,
    ) -> Result<TransactionStatus> {
        let client = self.rpc_client.lock().await;
        
        let config = RpcTransactionConfig {
            encoding: None,
            commitment: Some(self.commitment),
            max_supported_transaction_version: Some(0),
        };
        
        let response = client.get_transaction_with_config(signature, config)?;
        
        Ok(TransactionStatus {
            signature: signature.to_string(),
            status: if response.meta.as_ref().map_or(false, |m| m.err.is_none()) {
                "success".to_string()
            } else {
                "failed".to_string()
            },
            slot: response.slot,
            block_time: response.block_time,
            confirmation_status: response.meta.and_then(|m| m.confirmation_status),
            error: response.meta.and_then(|m| m.err.map(|e| format!("{:?}", e))),
        })
    }
    
    pub async fn get_account_data(
        &self,
        pubkey: &solana_sdk::pubkey::Pubkey,
    ) -> Result<Vec<u8>> {
        let client = self.rpc_client.lock().await;
        let account = client.get_account(pubkey)?;
        Ok(account.data)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub signature: String,
    pub status: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub confirmation_status: Option<String>,
    pub error: Option<String>,
}