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

#[derive(Accounts)]
pub struct CreateTransaction<'info> {
    #[account(
        mut,
        has_one = multisig
    )]
    pub transaction: Account<'info, Transaction>,

    #[account()]
    pub multisig: Account<'info, Multisig>,

    /// CHECK: only validated by position check in handler
    pub proposer: Signer<'info>,

    #[account()]
    pub system_program: Program<'info, System>,
}
