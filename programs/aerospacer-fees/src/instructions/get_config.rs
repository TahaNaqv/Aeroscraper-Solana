use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct GetConfig<'info> {
    pub state: Account<'info, FeeStateAccount>,
}

pub fn handler(ctx: Context<GetConfig>) -> Result<ConfigResponse> {
    let state = &ctx.accounts.state;
    
    let response = ConfigResponse {
        admin: state.admin,
        is_stake_enabled: state.is_stake_enabled,
        stake_contract_address: state.stake_contract_address,
        total_fees_collected: state.total_fees_collected,
    };
    
    msg!("Fee distributor config retrieved successfully");
    msg!("Admin: {}", response.admin);
    msg!("Stake enabled: {}", response.is_stake_enabled);
    msg!("Total fees collected: {}", response.total_fees_collected);
    
    Ok(response)
} 