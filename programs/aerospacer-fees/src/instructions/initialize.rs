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
    
    // Set default fee distribution addresses (equivalent to INJECTIVE's FEE_ADDR_1 and FEE_ADDR_2)
    // In production, these would be actual fee recipient addresses
    state.fee_address_1 = Pubkey::default(); // TODO: Set actual fee address 1
    state.fee_address_2 = Pubkey::default(); // TODO: Set actual fee address 2
    
    msg!("Aerospacer Fee Distributor initialized successfully");
    msg!("Admin: {}", state.admin);
    msg!("Stake enabled: {}", state.is_stake_enabled);
    msg!("Total fees collected: {}", state.total_fees_collected);
    
    Ok(())
}


