use anchor_lang::prelude::*;
use crate::state::FeeStateAccount;

#[derive(Accounts)]
pub struct ToggleStakeContract<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = state.admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub state: Account<'info, FeeStateAccount>,
}

pub fn handler(ctx: Context<ToggleStakeContract>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Toggle the stake enabled flag
    state.is_stake_enabled = !state.is_stake_enabled;
    
    msg!("Stake contract toggled successfully");
    msg!("Stake enabled: {}", state.is_stake_enabled);
    
    if state.is_stake_enabled {
        msg!("Fees will now be distributed to stability pool");
    } else {
        msg!("Fees will now be distributed to fee addresses");
    }
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
}
