use anchor_lang::prelude::*;
use crate::state::{SortedTrovesState, StateAccount};
use crate::error::AerospacerProtocolError;

#[derive(Accounts)]
pub struct ResetSortedTroves<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [b"sorted_troves_state"],
        bump,
    )]
    pub sorted_troves_state: Box<Account<'info, SortedTrovesState>>,

    #[account(
        mut,
        seeds = [b"state"],
        bump,
        constraint = state.admin == authority.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub state: Box<Account<'info, StateAccount>>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<ResetSortedTroves>) -> Result<()> {
    msg!("⚠️  ADMIN: Resetting corrupted sorted troves state");
    msg!("  Previous size: {}", ctx.accounts.sorted_troves_state.size);
    msg!("  Previous head: {:?}", ctx.accounts.sorted_troves_state.head);
    msg!("  Previous tail: {:?}", ctx.accounts.sorted_troves_state.tail);
    
    msg!("✅ SortedTrovesState account closed - will be reinitialized on next openTrove");
    msg!("⚠️  WARNING: All trove ordering has been reset!");
    msg!("   Existing troves will need to call openTrove again to re-insert into sorted list");
    
    Ok(())
}