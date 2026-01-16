use crate::contract::VaultContract;
use crate::db::Database;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use sqlx::PgPool;
use std::sync::Arc;

pub struct VaultManager {
    contract: Arc<VaultContract>,
    db: Database,
}

impl VaultManager {
    pub fn new(contract: Arc<VaultContract>, db_pool: PgPool) -> Self {
        Self {
            contract,
            db: Database::new(db_pool),
        }
    }
    
    pub async fn initialize_user_vault(&self, user_pubkey: &str) -> Result<String, VaultError> {
        let pubkey = Pubkey::from_str(user_pubkey)?;
        
        // Check if vault already exists
        if let Ok(_) = self.db.get_vault_by_owner(&user_pubkey).await {
            return Err(VaultError::VaultAlreadyExists);
        }
        
        // Initialize on-chain
        let signature = self.contract.initialize_vault(&pubkey).await?;
        
        // Store in database
        let (vault_pda, bump) = Pubkey::find_program_address(
            &[b"vault", pubkey.as_ref()],
            &self.contract.program.id(),
        );
        
        self.db.create_vault(
            user_pubkey,
            &vault_pda.to_string(),
            &get_associated_token_address(&vault_pda, &USDT_MINT).to_string(),
            bump,
        ).await?;
        
        Ok(format!("Vault initialized: {}", signature))
    }
    
    pub async fn deposit_collateral(
        &self,
        user_keypair: &Keypair,
        amount: u64,
        user_signature: &str,
    ) -> Result<String, VaultError> {
        // Verify user signature (simplified)
        if !self.verify_signature(user_keypair.pubkey(), user_signature) {
            return Err(VaultError::InvalidSignature);
        }
        
        // Get current balance for validation
        let vault_state = self.contract.get_vault_state(&user_keypair.pubkey()).await?;
        
        // Execute deposit
        let signature = self.contract.deposit(user_keypair, amount).await?;
        
        // Update database
        self.db.record_transaction(
            &user_keypair.pubkey().to_string(),
            "deposit",
            amount as i64,
            &signature.to_string(),
            None,
            None,
        ).await?;
        
        self.db.update_vault_balance(
            &user_keypair.pubkey().to_string(),
            amount as i64,
            0, // No change in locked
            amount as i64, // Increase available
        ).await?;
        
        Ok(format!("Deposit successful: {}", signature))
    }
    
    pub async fn withdraw_collateral(
        &self,
        user_keypair: &Keypair,
        amount: u64,
        user_signature: &str,
    ) -> Result<String, VaultError> {
        // Verify signature
        if !self.verify_signature(user_keypair.pubkey(), user_signature) {
            return Err(VaultError::InvalidSignature);
        }
        
        // Check available balance
        let vault_state = self.contract.get_vault_state(&user_keypair.pubkey()).await?;
        
        if vault_state.available_balance < amount {
            return Err(VaultError::InsufficientBalance);
        }
        
        // Execute withdrawal
        let signature = self.contract.withdraw(user_keypair, amount).await?;
        
        // Update database
        self.db.record_transaction(
            &user_keypair.pubkey().to_string(),
            "withdrawal",
            -(amount as i64), // Negative for withdrawal
            &signature.to_string(),
            None,
            None,
        ).await?;
        
        self.db.update_vault_balance(
            &user_keypair.pubkey().to_string(),
            -(amount as i64),
            0,
            -(amount as i64),
        ).await?;
        
        Ok(format!("Withdrawal successful: {}", signature))
    }
    
    pub async fn get_vault_balance(&self, user_pubkey: &str) -> Result<VaultBalance, VaultError> {
        let pubkey = Pubkey::from_str(user_pubkey)?;
        let vault_state = self.contract.get_vault_state(&pubkey).await?;
        
        Ok(VaultBalance {
            owner: user_pubkey.to_string(),
            total_balance: vault_state.total_balance,
            locked_balance: vault_state.locked_balance,
            available_balance: vault_state.available_balance,
            total_deposited: vault_state.total_deposited,
            total_withdrawn: vault_state.total_withdrawn,
        })
    }
    
    fn verify_signature(&self, pubkey: Pubkey, signature: &str) -> bool {
        // Implement actual signature verification
        // This is a simplified version
        true
    }
}