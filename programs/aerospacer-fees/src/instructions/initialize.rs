use anchor_lang::prelude::*;
use crate::state::FeeStateAccount;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + FeeStateAccount::LEN)]
    pub state: Account<'info, FeeStateAccount>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    state.admin = ctx.accounts.admin.key();
    state.is_stake_enabled = false; // Default to disabled
    state.stake_contract_address = Pubkey::default(); // Will be set later
    state.total_fees_collected = 0;
    
    msg!("Aerospacer Fee Distributor initialized successfully");
    msg!("Admin: {}", state.admin);
    msg!("Stake enabled: {}", state.is_stake_enabled);
    msg!("Total fees collected: {}", state.total_fees_collected);
    
    Ok(())
}


