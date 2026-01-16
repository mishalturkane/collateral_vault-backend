use solana_sdk::{
    pubkey::Pubkey,
    transaction::Transaction,
    instruction::Instruction,
    compute_budget::ComputeBudgetInstruction,
};
use anchor_client::{
    solana_sdk::{
        signature::Keypair,
        signer::Signer,
    },
};
use anyhow::{Result, Context};

pub struct TransactionBuilder {
    payer: Keypair,
    recent_blockhash: solana_sdk::hash::Hash,
    instructions: Vec<Instruction>,
    signers: Vec<Keypair>,
    priority_fee: Option<u64>,
}

impl TransactionBuilder {
    pub fn new(payer: Keypair, recent_blockhash: solana_sdk::hash::Hash) -> Self {
        Self {
            payer,
            recent_blockhash,
            instructions: Vec::new(),
            signers: vec![payer.clone()],
            priority_fee: None,
        }
    }
    
    pub fn add_instruction(mut self, instruction: Instruction) -> Self {
        self.instructions.push(instruction);
        self
    }
    
    pub fn add_signer(mut self, signer: Keypair) -> Self {
        self.signers.push(signer);
        self
    }
    
    pub fn set_priority_fee(mut self, micro_lamports: u64) -> Self {
        self.priority_fee = Some(micro_lamports);
        self
    }
    
    pub fn build(self) -> Result<Transaction> {
        let mut instructions = self.instructions;
        
        // Add priority fee instruction if specified
        if let Some(micro_lamports) = self.priority_fee {
            let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(micro_lamports);
            instructions.insert(0, priority_fee_ix);
        }
        
        let mut transaction = Transaction::new_with_payer(
            &instructions,
            Some(&self.payer.pubkey()),
        );
        
        transaction.sign(&self.signers, self.recent_blockhash);
        
        Ok(transaction)
    }
}