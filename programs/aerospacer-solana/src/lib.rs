use anchor_lang::prelude::*;

declare_id!("6kJZg8PDkutRui282AnspEnLcyExxcpsbCvyfBoTcDwN");

#[program]
pub mod aerospacer_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
