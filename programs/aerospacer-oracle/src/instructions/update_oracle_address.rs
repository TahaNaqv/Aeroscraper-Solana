use anchor_lang::prelude::*;
use crate::state::OracleStateAccount;
use crate::error::AerospacerOracleError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateOracleAddressParams {
    pub new_oracle_address: Pubkey,
}

#[derive(Accounts)]
#[instruction(params: UpdateOracleAddressParams)]
pub struct UpdateOracleAddress<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = state.admin == admin.key() @ AerospacerOracleError::Unauthorized
    )]
    pub state: Account<'info, OracleStateAccount>,
}

pub fn handler(ctx: Context<UpdateOracleAddress>, params: UpdateOracleAddressParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Update the oracle address
    state.oracle_address = params.new_oracle_address;
    
    msg!("Oracle address updated successfully");
    msg!("New oracle address: {}", params.new_oracle_address);
    msg!("Updated by admin: {}", ctx.accounts.admin.key());
    
    Ok(())
}
