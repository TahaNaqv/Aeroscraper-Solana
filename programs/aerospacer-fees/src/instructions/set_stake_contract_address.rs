use anchor_lang::prelude::*;
use std::str::FromStr;
use crate::state::FeeStateAccount;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetStakeContractAddressParams {
    pub address: String, // Equivalent to INJECTIVE's address: String
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
    
    // Validate and set the stake contract address (equivalent to INJECTIVE's addr_validate)
    // In Solana, we'll convert the string to a Pubkey
    let stake_contract_address = match Pubkey::try_from(params.address.as_str()) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(ErrorCode::InvalidAddress.into()),
    };
    
    state.stake_contract_address = stake_contract_address;
    
    msg!("Stake contract address set successfully");
    msg!("New address: {}", stake_contract_address);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid address")]
    InvalidAddress,
}
