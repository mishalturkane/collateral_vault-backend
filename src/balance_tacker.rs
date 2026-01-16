use crate::db::Database;
use crate::contract::VaultContract;
use std::sync::Arc;
use tokio::time::{self, Duration};
use log::{info, warn, error};

pub struct BalanceTracker {
    db: Database,
    contract: Arc<VaultContract>,
    interval_seconds: u64,
}

impl BalanceTracker {
    pub fn new(db: Database, contract: Arc<VaultContract>, interval_seconds: u64) -> Self {
        Self {
            db,
            contract,
            interval_seconds,
        }
    }
    
    pub async fn start(&self) {
        let mut interval = time::interval(Duration::from_secs(self.interval_seconds));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.reconcile_all_vaults().await {
                error!("Reconciliation error: {}", e);
            }
        }
    }
    
    async fn reconcile_all_vaults(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting vault reconciliation...");
        
        // Get all vaults from database
        // This is simplified - you need to implement get_all_vaults in Database
        let vaults = self.get_all_vaults_from_db().await?;
        
        let mut total_discrepancies = 0;
        
        for vault in vaults {
            match self.reconcile_vault(&vault).await {
                Ok(discrepancy) => {
                    if discrepancy != 0 {
                        warn!("Discrepancy found for vault {}: {}", vault.owner, discrepancy);
                        total_discrepancies += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to reconcile vault {}: {}", vault.owner, e);
                }
            }
        }
        
        info!("Reconciliation complete. Found {} discrepancies", total_discrepancies);
        Ok(())
    }
    
    async fn reconcile_vault(&self, vault: &VaultRecord) -> Result<i64, Box<dyn std::error::Error>> {
        // Get on-chain balance
        let pubkey = Pubkey::from_str(&vault.owner)?;
        let onchain_state = self.contract.get_vault_state(&pubkey).await?;
        
        // Calculate discrepancy
        let onchain_total = onchain_state.total_balance as i64;
        let db_total = vault.total_balance;
        
        let discrepancy = onchain_total - db_total;
        
        if discrepancy != 0 {
            // Log discrepancy
            self.log_discrepancy(&vault.owner, onchain_total, db_total, discrepancy).await?;
            
            // Auto-correct if discrepancy is small (threshold)
            if discrepancy.abs() < 1000 { // 0.001 USDT
                self.db.update_vault_balance(
                    &vault.owner,
                    discrepancy,
                    0,
                    discrepancy,
                ).await?;
                info!("Auto-corrected vault {}: {}", vault.owner, discrepancy);
            }
        }
        
        Ok(discrepancy)
    }
    
    async fn log_discrepancy(
        &self,
        owner: &str,
        onchain_balance: i64,
        offchain_balance: i64,
        discrepancy: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO reconciliation_logs 
            (vault_owner, onchain_balance, offchain_balance, discrepancy, reconciled_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
            owner,
            onchain_balance,
            offchain_balance,
            discrepancy,
        )
        .execute(&self.db.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_all_vaults_from_db(&self) -> Result<Vec<VaultRecord>, sqlx::Error> {
        // Implement this method to fetch all vaults
        // Simplified placeholder
        Ok(vec![])
    }
}