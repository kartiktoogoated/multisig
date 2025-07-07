use anchor_lang::prelude::*;

#[account]
pub struct Multisig {
    pub owners: Vec<Pubkey>,
    pub threshold: u8,
    pub nonce: u8,
    pub owner_set_seqno: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransactionAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&TransactionAccount> for anchor_lang::solana_program::instruction::AccountMeta {
    fn from(account: &TransactionAccount) -> Self {
        if account.is_writable {
            anchor_lang::solana_program::instruction::AccountMeta::new(account.pubkey, account.is_signer)
        } else {
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(account.pubkey, account.is_signer)
        }
    }
}

#[account]
pub struct Transaction {
    pub multisig: Pubkey,                   // The parent multisig account
    pub program_id: Pubkey,                 // Target program to call
    pub accounts: Vec<TransactionAccount>,  // Account metas for the instruction
    pub data: Vec<u8>,                      // Serialized instruction data
    pub signers: Vec<bool>,                 // Track which multisig owners signed
    pub did_execute: bool,                  // Has this been executed?
    pub owner_set_seqno: u32,               // Same seqno from multisig
}