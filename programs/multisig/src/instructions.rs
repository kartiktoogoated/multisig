use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateMultisig<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [b"multisig", payer.key().as_ref()],
        bump,
        space = 8 + 4 + 32 * 10 + 1 + 1 + 4,
    )]
    pub multisig: Account<'info, Multisig>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Helper function
pub fn assert_unique_owners(owners: &[Pubkey]) -> Result<()> {
    for (i, owner) in owners.iter().enumerate() {
        require!(
            !owners.iter().skip(i + 1).any(|item| item == owner),
            MultisigError::DuplicateOwners
        );
    }
    Ok(())
}

// Easier logic for assert unique owners
/*
for i in 0..owners.len() {
    let owner = owners[i];
    for j in (i+1)..owners.len() {
        if owners[j] == owner {
            throw DuplicateOwnersError
        }
    }
}
*/

#[derive(Accounts)]
pub struct ProposeTransaction<'info> {
    #[account(
        init,
        payer = proposer,
        space = 1000
    )]
    pub transaction: Account<'info, Transaction>,

    #[account()]
    pub multisig: Account<'info, Multisig>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account()]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveTransaction<'info> {
    #[account(
        mut,
        has_one = multisig
    )]
    pub transaction: Account<'info, Transaction>,

    pub multisig: Account<'info, Multisig>,

    pub owner: Signer<'info>,
}
#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
    #[account(
        mut,
        has_one = multisig,
    )]
    pub transaction: Account<'info, Transaction>,

    pub multisig: Account<'info, Multisig>,

    /// CHECK: this PDA will sign the CPI we dont read write its data
    #[account(
        seeds = [b"multisig-signer", multisig.key().as_ref()],
        bump = multisig.nonce
    )]
    pub multisig_signer: UncheckedAccount<'info>,
}
