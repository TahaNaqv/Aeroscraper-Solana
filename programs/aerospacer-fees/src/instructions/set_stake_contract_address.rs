use anchor_lang::prelude::*;
use crate::state::FeeStateAccount;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetStakeContractAddressParams {
    pub address: Pubkey,
}

#[derive(Accounts)]
#[instruction(params: SetStakeContractAddressParams)]
pub struct SetStakeContractAddress<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = state.admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub state: Account<'info, FeeStateAccount>,
}

pub fn handler(ctx: Context<SetStakeContractAddress>, params: SetStakeContractAddressParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Set the stake contract address
    state.stake_contract_address = params.address;
    
    msg!("Stake contract address set successfully");
    msg!("New address: {}", params.address);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid address")]
    InvalidAddress,
}
