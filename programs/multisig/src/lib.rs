#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("CM6qMaBEnGu9ZEZhF88YZgsLR7wQvZ2C167YvuznCi5P");

pub mod error;
pub mod instructions;
pub mod state;

use error::*;
use state::*;
use instructions::*;

#[program]
pub mod multisig {
    use super::*;

    pub fn create_multisig(
        ctx: Context<CreateMultisig>,
        owners: Vec<Pubkey>,
        threshold: u8,
        nonce: u8,
    ) -> Result<()> {
        assert_unique_owners(&owners)?;

        require!(!owners.is_empty(), MultisigError::NoOwners);
        require!(
            threshold > 0 && threshold <= owners.len() as u8,
            MultisigError::InvalidThreshold
        );

        let multisig = &mut ctx.accounts.multisig;
        multisig.owners = owners;
        multisig.threshold = threshold;
        multisig.nonce = nonce;
        multisig.owner_set_seqno = 0;

        Ok(())
    }

    pub fn create_transaction(
        ctx: Context<CreateTransaction>,
        program_id: Pubkey,
        accounts: Vec<TransactionAccount>,
        data: Vec<u8>,
    ) -> Result<()> {
        let multisig = &ctx.accounts.multisig;
        let proposer_key = ctx.accounts.proposer.key();

        // Find index of proposer in owners
        let owner_index = multisig
            .owners
            .iter()
            .position(|x| x == &proposer_key)
            .ok_or(MultisigError::InvalidOwner)?;

        // Initialize signer flags
        let mut signers = vec![false; multisig.owners.len()];
        signers[owner_index] = true;

        // Save transaction
        let tx = &mut ctx.accounts.transaction;
        tx.multisig = multisig.key();
        tx.program_id = program_id;
        tx.accounts = accounts;
        tx.data = data;
        tx.signers = signers;
        tx.did_execute = false;
        tx.owner_set_seqno = multisig.owner_set_seqno;

        Ok(())
    }
}
