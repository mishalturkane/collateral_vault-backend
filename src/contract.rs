use anchor_client::{
    anchor_lang::system_program,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::Transaction,
    },
    Program,
};
use anchor_lang::prelude::*;
use std::str::FromStr;

pub struct VaultContract {
    program: Program,
    rpc_client: RpcClient,
    payer: Keypair,
}

impl VaultContract {
    pub fn new(rpc_url: &str, payer: Keypair) -> Self {
        let program_id = Pubkey::from_str("G6TF8EdpP7gKwfPmNEhMLU7E34X5Fr3ujpAMdCzwHz8R")
            .expect("Invalid program ID");
        
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );
        
        let program = Program::new(
            program_id,
            &rpc_client,
            payer.clone(),
        );
        
        Self {
            program,
            rpc_client,
            payer,
        }
    }
    
    pub async fn initialize_vault(&self, user: &Pubkey) -> Result<Signature, Box<dyn std::error::Error>> {
        let (vault_pda, _bump) = Pubkey::find_program_address(
            &[b"vault", user.as_ref()],
            &self.program.id(),
        );
        
        let tx = self.program
            .request()
            .accounts(collateral_vault::accounts::InitializeVault {
                user: *user,
                token_mint: Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")?, // USDT mint
                vault: vault_pda,
                user_token_account: get_associated_token_address(user, &USDT_MINT),
                vault_token_account: get_associated_token_address(&vault_pda, &USDT_MINT),
                token_program: anchor_spl::token::ID,
                associated_token_program: anchor_spl::associated_token::ID,
                system_program: system_program::ID,
            })
            .args(collateral_vault::instruction::InitializeVault {})
            .signer(&self.payer)
            .send()
            .await?;
        
        Ok(tx)
    }
    
    pub async fn deposit(
        &self,
        user: &Keypair,
        amount: u64,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let (vault_pda, _bump) = Pubkey::find_program_address(
            &[b"vault", user.pubkey().as_ref()],
            &self.program.id(),
        );
        
        let tx = self.program
            .request()
            .accounts(collateral_vault::accounts::Deposit {
                user: user.pubkey(),
                vault: vault_pda,
                token_mint: USDT_MINT,
                user_token_account: get_associated_token_address(&user.pubkey(), &USDT_MINT),
                vault_token_account: get_associated_token_address(&vault_pda, &USDT_MINT),
                token_program: anchor_spl::token::ID,
            })
            .args(collateral_vault::instruction::Deposit { amount })
            .signer(user)
            .send()
            .await?;
        
        Ok(tx)
    }
    
    pub async fn withdraw(
        &self,
        user: &Keypair,
        amount: u64,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let (vault_pda, _bump) = Pubkey::find_program_address(
            &[b"vault", user.pubkey().as_ref()],
            &self.program.id(),
        );
        
        let tx = self.program
            .request()
            .accounts(collateral_vault::accounts::Withdraw {
                user: user.pubkey(),
                vault: vault_pda,
                token_mint: USDT_MINT,
                user_token_account: get_associated_token_address(&user.pubkey(), &USDT_MINT),
                vault_token_account: get_associated_token_address(&vault_pda, &USDT_MINT),
                token_program: anchor_spl::token::ID,
            })
            .args(collateral_vault::instruction::Withdraw { amount })
            .signer(user)
            .send()
            .await?;
        
        Ok(tx)
    }
    
    pub async fn get_vault_state(&self, user: &Pubkey) -> Result<collateral_vault::CollateralVault, Box<dyn std::error::Error>> {
        let (vault_pda, _bump) = Pubkey::find_program_address(
            &[b"vault", user.as_ref()],
            &self.program.id(),
        );
        
        let account = self.rpc_client.get_account_data(&vault_pda)?;
        let vault_state: collateral_vault::CollateralVault = AccountDeserialize::try_deserialize(&mut &account[8..])?;
        
        Ok(vault_state)
    }
}