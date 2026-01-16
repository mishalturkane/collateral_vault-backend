use std::str::FromStr;
use std::sync::Arc;
use anchor_client::{
    Client, Cluster,
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::Transaction,
        commitment_config::CommitmentConfig,
    },
};
use anchor_lang::prelude::AccountMeta;
use anyhow::{Result, Context};
use serde_json::Value;
use tracing::{info, warn, error};

use crate::database::DatabasePool;
use crate::services::rpc::RpcService;
use crate::utils::anchor_client::AnchorClient;
use crate::models::{
    requests::*,
    responses::*,
    database::{Vault, VaultEvent},
};

#[derive(Clone)]
pub struct VaultService {
    db_pool: DatabasePool,
    rpc_service: RpcService,
    anchor_client: AnchorClient,
    admin_keypair: Keypair,
}

impl VaultService {
    pub fn new(
        db_pool: DatabasePool,
        rpc_service: RpcService,
        program_id: String,
        admin_keypair_path: std::path::PathBuf,
    ) -> Result<Self> {
        let admin_keypair = Keypair::from_base58_string(
            &std::fs::read_to_string(admin_keypair_path)?
        )?;
        
        let anchor_client = AnchorClient::new(
            program_id,
            admin_keypair.clone(),
            rpc_service.clone(),
        )?;
        
        Ok(Self {
            db_pool,
            rpc_service,
            anchor_client,
            admin_keypair,
        })
    }
    
    pub async fn initialize_vault(
        &self,
        owner: &str,
        token_mint: &str,
    ) -> Result<InitializeVaultResult> {
        let owner_pubkey = Pubkey::from_str(owner)?;
        let token_mint_pubkey = Pubkey::from_str(token_mint)?;
        
        // Build transaction using Anchor client
        let tx = self.anchor_client.build_initialize_vault_transaction(
            owner_pubkey,
            token_mint_pubkey,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        // Wait for confirmation
        self.rpc_service.confirm_transaction(&signature).await?;
        
        // Store vault in database
        let vault_pubkey = self.anchor_client.get_vault_pda(owner_pubkey)?;
        
        self.db_pool.store_vault(Vault {
            id: uuid::Uuid::new_v4(),
            owner: owner.to_string(),
            vault_address: vault_pubkey.to_string(),
            token_mint: token_mint.to_string(),
            total_balance: 0,
            locked_balance: 0,
            available_balance: 0,
            total_deposited: 0,
            total_withdrawn: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }).await?;
        
        Ok(InitializeVaultResult {
            vault_address: vault_pubkey.to_string(),
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
        })
    }
    
    pub async fn deposit_collateral(
        &self,
        owner: &str,
        amount: u64,
        user_token_account: &str,
        priority_fee: Option<u64>,
    ) -> Result<TransactionResult> {
        let owner_pubkey = Pubkey::from_str(owner)?;
        let user_token_account_pubkey = Pubkey::from_str(user_token_account)?;
        
        // Get vault PDA
        let vault_pubkey = self.anchor_client.get_vault_pda(owner_pubkey)?;
        
        // Build deposit transaction
        let tx = self.anchor_client.build_deposit_transaction(
            owner_pubkey,
            vault_pubkey,
            user_token_account_pubkey,
            amount,
            priority_fee,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        // Log event
        self.log_vault_event(
            &owner,
            "deposit",
            &serde_json::json!({
                "amount": amount,
                "signature": signature.to_string(),
            }),
        ).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn withdraw_collateral(
        &self,
        owner: &str,
        amount: u64,
        user_token_account: &str,
        priority_fee: Option<u64>,
    ) -> Result<TransactionResult> {
        let owner_pubkey = Pubkey::from_str(owner)?;
        let user_token_account_pubkey = Pubkey::from_str(user_token_account)?;
        
        // Get vault PDA
        let vault_pubkey = self.anchor_client.get_vault_pda(owner_pubkey)?;
        
        // Build withdraw transaction
        let tx = self.anchor_client.build_withdraw_transaction(
            owner_pubkey,
            vault_pubkey,
            user_token_account_pubkey,
            amount,
            priority_fee,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        // Log event
        self.log_vault_event(
            &owner,
            "withdraw",
            &serde_json::json!({
                "amount": amount,
                "signature": signature.to_string(),
            }),
        ).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn lock_collateral(
        &self,
        owner: &str,
        amount: u64,
        caller_program: &str,
        priority_fee: Option<u64>,
    ) -> Result<TransactionResult> {
        let owner_pubkey = Pubkey::from_str(owner)?;
        let caller_program_pubkey = Pubkey::from_str(caller_program)?;
        
        // Get vault PDA
        let vault_pubkey = self.anchor_client.get_vault_pda(owner_pubkey)?;
        
        // Build lock transaction
        let tx = self.anchor_client.build_lock_collateral_transaction(
            vault_pubkey,
            caller_program_pubkey,
            amount,
            priority_fee,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        // Update database
        self.db_pool.update_vault_balances(
            owner,
            |vault| {
                if vault.available_balance >= amount {
                    vault.available_balance -= amount;
                    vault.locked_balance += amount;
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Insufficient available balance"))
                }
            },
        ).await?;
        
        // Log event
        self.log_vault_event(
            &owner,
            "lock",
            &serde_json::json!({
                "amount": amount,
                "caller_program": caller_program,
                "signature": signature.to_string(),
            }),
        ).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn unlock_collateral(
        &self,
        owner: &str,
        amount: u64,
        caller_program: &str,
        priority_fee: Option<u64>,
    ) -> Result<TransactionResult> {
        let owner_pubkey = Pubkey::from_str(owner)?;
        let caller_program_pubkey = Pubkey::from_str(caller_program)?;
        
        // Get vault PDA
        let vault_pubkey = self.anchor_client.get_vault_pda(owner_pubkey)?;
        
        // Build unlock transaction
        let tx = self.anchor_client.build_unlock_collateral_transaction(
            vault_pubkey,
            caller_program_pubkey,
            amount,
            priority_fee,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        // Update database
        self.db_pool.update_vault_balances(
            owner,
            |vault| {
                if vault.locked_balance >= amount {
                    vault.locked_balance -= amount;
                    vault.available_balance += amount;
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Insufficient locked balance"))
                }
            },
        ).await?;
        
        // Log event
        self.log_vault_event(
            &owner,
            "unlock",
            &serde_json::json!({
                "amount": amount,
                "caller_program": caller_program,
                "signature": signature.to_string(),
            }),
        ).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn transfer_collateral(
        &self,
        from_owner: &str,
        to_owner: &str,
        amount: u64,
        caller_program: &str,
        priority_fee: Option<u64>,
    ) -> Result<TransactionResult> {
        let from_owner_pubkey = Pubkey::from_str(from_owner)?;
        let to_owner_pubkey = Pubkey::from_str(to_owner)?;
        let caller_program_pubkey = Pubkey::from_str(caller_program)?;
        
        // Get vault PDAs
        let from_vault_pubkey = self.anchor_client.get_vault_pda(from_owner_pubkey)?;
        let to_vault_pubkey = self.anchor_client.get_vault_pda(to_owner_pubkey)?;
        
        // Build transfer transaction
        let tx = self.anchor_client.build_transfer_collateral_transaction(
            from_vault_pubkey,
            to_vault_pubkey,
            caller_program_pubkey,
            amount,
            priority_fee,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        // Update both vaults in database
        self.db_pool.update_vault_balances(
            from_owner,
            |vault| {
                if vault.available_balance >= amount {
                    vault.available_balance -= amount;
                    vault.total_balance -= amount;
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Insufficient available balance"))
                }
            },
        ).await?;
        
        self.db_pool.update_vault_balances(
            to_owner,
            |vault| {
                vault.available_balance += amount;
                vault.total_balance += amount;
                Ok(())
            },
        ).await?;
        
        // Log event
        self.log_vault_event(
            from_owner,
            "transfer_out",
            &serde_json::json!({
                "amount": amount,
                "to_owner": to_owner,
                "caller_program": caller_program,
                "signature": signature.to_string(),
            }),
        ).await?;
        
        self.log_vault_event(
            to_owner,
            "transfer_in",
            &serde_json::json!({
                "amount": amount,
                "from_owner": from_owner,
                "caller_program": caller_program,
                "signature": signature.to_string(),
            }),
        ).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn close_vault(&self, owner: &str) -> Result<TransactionResult> {
        let owner_pubkey = Pubkey::from_str(owner)?;
        
        // Get vault PDA
        let vault_pubkey = self.anchor_client.get_vault_pda(owner_pubkey)?;
        
        // Build close vault transaction
        let tx = self.anchor_client.build_close_vault_transaction(
            owner_pubkey,
            vault_pubkey,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        // Delete vault from database
        self.db_pool.delete_vault(owner).await?;
        
        // Log event
        self.log_vault_event(
            &owner,
            "close",
            &serde_json::json!({
                "signature": signature.to_string(),
            }),
        ).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn initialize_authority(
        &self,
        authorized_programs: &[String],
        priority_fee: Option<u64>,
    ) -> Result<TransactionResult> {
        let authorized_program_pubkeys: Vec<Pubkey> = authorized_programs
            .iter()
            .map(|p| Pubkey::from_str(p))
            .collect::<Result<Vec<_>, _>>()?;
        
        let tx = self.anchor_client.build_initialize_authority_transaction(
            &authorized_program_pubkeys,
            priority_fee,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn add_authorized_program(
        &self,
        program: &str,
        priority_fee: Option<u64>,
    ) -> Result<TransactionResult> {
        let program_pubkey = Pubkey::from_str(program)?;
        
        let tx = self.anchor_client.build_add_authorized_program_transaction(
            program_pubkey,
            priority_fee,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn remove_authorized_program(
        &self,
        program: &str,
    ) -> Result<TransactionResult> {
        let program_pubkey = Pubkey::from_str(program)?;
        
        let tx = self.anchor_client.build_remove_authorized_program_transaction(
            program_pubkey,
        ).await?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        Ok(TransactionResult {
            transaction: bs58::encode(tx.message_data()).into_string(),
            signature: signature.to_string(),
            estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
        })
    }
    
    pub async fn get_vault_info(&self, owner: &str) -> Result<VaultInfo> {
        let vault = self.db_pool.get_vault(owner).await?;
        
        Ok(VaultInfo {
            owner: vault.owner,
            vault_address: vault.vault_address,
            total_balance: vault.total_balance,
            locked_balance: vault.locked_balance,
            available_balance: vault.available_balance,
            total_deposited: vault.total_deposited,
            total_withdrawn: vault.total_withdrawn,
            created_at: vault.created_at,
            token_mint: vault.token_mint,
        })
    }
    
    pub async fn build_transaction(
        &self,
        tx_type: &str,
        parameters: &Value,
        priority_fee: Option<u64>,
    ) -> Result<TransactionResult> {
        match tx_type {
            "initialize_vault" => {
                let owner = parameters["owner"].as_str().unwrap();
                let token_mint = parameters["token_mint"].as_str().unwrap();
                let owner_pubkey = Pubkey::from_str(owner)?;
                let token_mint_pubkey = Pubkey::from_str(token_mint)?;
                
                let tx = self.anchor_client.build_initialize_vault_transaction(
                    owner_pubkey,
                    token_mint_pubkey,
                ).await?;
                
                Ok(TransactionResult {
                    transaction: bs58::encode(tx.message_data()).into_string(),
                    signature: "".to_string(), // Not signed yet
                    estimated_fee: self.rpc_service.get_fee_for_transaction(&tx).await?,
                })
            }
            // Add other transaction types...
            _ => Err(anyhow::anyhow!("Unknown transaction type: {}", tx_type)),
        }
    }
    
    pub async fn submit_transaction(
        &self,
        signed_transaction: &str,
    ) -> Result<TransactionStatus> {
        let tx_data = bs58::decode(signed_transaction).into_vec()?;
        let tx = Transaction::try_from(&tx_data[..])?;
        
        let signature = self.rpc_service.send_transaction(&tx).await?;
        
        let status = self.rpc_service.get_transaction_status(&signature).await?;
        
        Ok(status)
    }
    
    pub async fn get_transaction_status(
        &self,
        signature: &str,
    ) -> Result<TransactionStatus> {
        let sig = Signature::from_str(signature)?;
        self.rpc_service.get_transaction_status(&sig).await
    }
    
    pub async fn stream_events(&self) -> impl futures::Stream<Item = Result<axum::response::sse::Event>> {
        // Implement SSE stream for real-time vault events
        use futures::stream::{self, StreamExt};
        use tokio::time::{interval, Duration};
        
        let mut interval = interval(Duration::from_secs(1));
        
        stream::unfold((), move |_| {
            let interval = interval.tick();
            async move {
                interval.await;
                Some((Ok(Event::default().data("ping")), ()))
            }
        })
    }
    
    async fn log_vault_event(
        &self,
        owner: &str,
        event_type: &str,
        data: &Value,
    ) -> Result<()> {
        let event = VaultEvent {
            id: uuid::Uuid::new_v4(),
            vault_owner: owner.to_string(),
            event_type: event_type.to_string(),
            data: data.clone(),
            created_at: chrono::Utc::now(),
        };
        
        self.db_pool.store_vault_event(event).await?;
        
        Ok(())
    }
}