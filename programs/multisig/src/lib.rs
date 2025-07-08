#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;

declare_id!("CM6qMaBEnGu9ZEZhF88YZgsLR7wQvZ2C167YvuznCi5P");

pub mod error;
pub mod instructions;
pub mod state;

use error::*;
use state::*;
use instructions::*;

#[program]
pub mod multisig {
    use anchor_lang::solana_program;

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

    pub fn propose_transaction(
        ctx: Context<ProposeTransaction>,
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

    pub fn approve_transaction(ctx: Context<ApproveTransaction>) -> Result<()> {
        let multisig = &ctx.accounts.multisig;
        let tx = &mut ctx.accounts.transaction;
        let owner_key = ctx.accounts.owner.key();

        // Check if owner is valid
        require!(
            tx.owner_set_seqno == multisig.owner_set_seqno,
            MultisigError::OwnerSetChanged
        );

        // Find index of approver
        let owner_index = multisig
            .owners
            .iter()
            .position(|pk| pk == &owner_key)
            .ok_or(MultisigError::InvalidOwner)?;

        // Mark signer approval
        tx.signers[owner_index] = true;

        Ok(())
    }

    pub fn execute_transaction(ctx: Context<ExecuteTransaction>) -> Result<()> {
        let tx = &mut ctx.accounts.transaction;
        let multisig = &ctx.accounts.multisig;

        // Prevent re execution
        require!(!tx.did_execute, MultisigError::AlreadyExecuted);

        // Ensure enough approvals
        let signed_count = tx.signers.iter().filter(|s| **s).count() as u8;
        require!(
            signed_count >= multisig.threshold,
            MultisigError::NotEnoughSigners
        );

        // Rebuild AccountMetas manually (no impl right now)
        let account_metas: Vec<AccountMeta> = tx.accounts.iter().map(|acc| {
            if acc.is_writable{
                AccountMeta::new(acc.pubkey, acc.is_signer)
            } else {
                AccountMeta::new_readonly(acc.pubkey, acc.is_signer)
            }
        }).collect();

        // Rebuild the Instruction
        let instruction = Instruction {
            program_id: tx.program_id,
            accounts: account_metas,
            data: tx.data.clone(),
        };

        // Derive the PDA signer seeds 
        let multisig_key = multisig.key();
        let signer_seeds: &[&[u8]] = &[
            b"multisig-signer",
            multisig_key.as_ref(),
            &[multisig.nonce],
        ];

        // Call the actual CPI
        solana_program::program::invoke_signed(
            &instruction,
            ctx.remaining_accounts,
            &[signer_seeds],
        )?;

        // Mark as executed
        tx.did_execute = true;

        Ok(())
    }
}
