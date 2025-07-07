use anchor_lang::prelude::*;

declare_id!("CM6qMaBEnGu9ZEZhF88YZgsLR7wQvZ2C167YvuznCi5P");

#[program]
pub mod multisig {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
