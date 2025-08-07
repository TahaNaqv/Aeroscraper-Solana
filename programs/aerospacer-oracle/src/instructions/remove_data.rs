use anchor_lang::prelude::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveDataParams {
    pub collateral_denom: String,
}

#[derive(Accounts)]
#[instruction(params: RemoveDataParams)]
pub struct RemoveData<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = state.admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub state: Account<'info, OracleStateAccount>,
}

pub fn handler(ctx: Context<RemoveData>, params: RemoveDataParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Find and remove the collateral data
    if let Some(index) = state.collateral_data.iter().position(|d| d.denom == params.collateral_denom) {
        state.collateral_data.remove(index);
        msg!("Removed collateral data for: {}", params.collateral_denom);
    } else {
        return Err(ErrorCode::CollateralDataNotFound.into());
    }
    
    msg!("Remove data successful");
    msg!("Removed denom: {}", params.collateral_denom);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Collateral data not found")]
    CollateralDataNotFound,
} 