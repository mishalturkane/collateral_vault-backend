use std::str::FromStr;
use anchor_client::{
    Client, Cluster,
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
        commitment_config::CommitmentConfig,
        instruction::Instruction,
    },
    anchor_lang::AnchorDeserialize,
};
use anyhow::{Result, Context};
use crate::services::transaction::TransactionBuilder;

#[derive(Clone)]
pub struct AnchorClient {
    program_id: Pubkey,
    admin_keypair: Keypair,
    rpc_url: String,
}

impl AnchorClient {
    pub fn new(
        program_id: String,
        admin_keypair: Keypair,
        rpc_url: String,
    ) -> Result<Self> {
        let program_id = Pubkey::from_str(&program_id)?;
        
        Ok(Self {
            program_id,
            admin_keypair,
            rpc_url,
        })
    }
    
    pub fn get_vault_pda(&self, owner: Pubkey) -> Result<Pubkey> {
        let (vault_pda, _bump) = Pubkey::find_program_address(
            &[b"vault", owner.as_ref()],
            &self.program_id,
        );
        
        Ok(vault_pda)
    }
    
    pub fn get_authority_pda(&self) -> Result<Pubkey> {
        let (authority_pda, _bump) = Pubkey::find_program_address(
            &[b"vault_authority"],
            &self.program_id,
        );
        
        Ok(authority_pda)
    }
    
    pub async fn build_initialize_vault_transaction(
        &self,
        owner: Pubkey,
        token_mint: Pubkey,
    ) -> Result<Transaction> {
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"vault", owner.as_ref()],
            &self.program_id,
        );
        
        let (vault_token_account, _) = Pubkey::find_program_address(
            &[
                vault_pda.as_ref(),
                &anchor_spl::token::ID.as_ref(),
                token_mint.as_ref(),
            ],
            &spl_associated_token_account::id(),
        );
        
        let user_token_account = spl_associated_token_account::get_associated_token_address(
            &owner,
            &token_mint,
        );
        
        let instruction_data = vec![
            0, // discriminator for initialize_vault
        ];
        
        let accounts = vec![
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(user_token_account, false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new_readonly(anchor_spl::token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ];
        
        let instruction = Instruction::new_with_bytes(
            self.program_id,
            &instruction_data,
            accounts,
        );
        
        let client = Client::new(
            Cluster::Custom(self.rpc_url.clone(), self.rpc_url.clone()),
            &self.admin_keypair,
        );
        
        let mut builder = TransactionBuilder::new(
            self.admin_keypair.clone(),
            client.get_latest_blockhash()?,
        );
        
        let tx = builder
            .add_instruction(instruction)
            .build()?;
        
        Ok(tx)
    }
    
    pub async fn build_deposit_transaction(
        &self,
        owner: Pubkey,
        vault: Pubkey,
        user_token_account: Pubkey,
        amount: u64,
        priority_fee: Option<u64>,
    ) -> Result<Transaction> {
        let (vault_pda, _) = Pubkey::find_program_address(
            &[b"vault", owner.as_ref()],
            &self.program_id,
        );
        
        let vault_token_account = spl_associated_token_account::get_associated_token_address(
            &vault_pda,
            &token_mint,
        );
        
        let instruction_data = vec![
            2, // discriminator for deposit
            (amount >> 0) as u8,
            (amount >> 8) as u8,
            (amount >> 16) as u8,
            (amount >> 24) as u8,
            (amount >> 32) as u8,
            (amount >> 40) as u8,
            (amount >> 48) as u8,
            (amount >> 56) as u8,
        ];
        
        let accounts = vec![
            AccountMeta::new(owner, true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new(user_token_account, false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new_readonly(anchor_spl::token::ID, false),
        ];
        
        let instruction = Instruction::new_with_bytes(
            self.program_id,
            &instruction_data,
            accounts,
        );
        
        let client = Client::new(
            Cluster::Custom(self.rpc_url.clone(), self.rpc_url.clone()),
            &self.admin_keypair,
        );
        
        let mut builder = TransactionBuilder::new(
            self.admin_keypair.clone(),
            client.get_latest_blockhash()?,
        );
        
        if let Some(fee) = priority_fee {
            builder = builder.set_priority_fee(fee);
        }
        
        let tx = builder
            .add_instruction(instruction)
            .build()?;
        
        Ok(tx)
    }
    
    // Similar methods for other transactions (withdraw, lock, unlock, transfer, etc.)
}