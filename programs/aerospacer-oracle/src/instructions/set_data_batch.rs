use anchor_lang::prelude::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetDataBatchParams {
    pub data: Vec<CollateralData>,
}

#[derive(Accounts)]
#[instruction(params: SetDataBatchParams)]
pub struct SetDataBatch<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = state.admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub state: Account<'info, OracleStateAccount>,
}

pub fn handler(ctx: Context<SetDataBatch>, params: SetDataBatchParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    let data_len = params.data.len();
    
    // Process each collateral data entry
    for collateral_data in params.data {
        // Check if denom already exists and update, otherwise add new
        if let Some(index) = state.collateral_data.iter().position(|d| d.denom == collateral_data.denom) {
            state.collateral_data[index] = collateral_data.clone();
            msg!("Updated collateral data for: {}", collateral_data.denom);
        } else {
            state.collateral_data.push(collateral_data.clone());
            msg!("Added new collateral data for: {}", collateral_data.denom);
        }
    }
    
    msg!("Set data batch successful");
    msg!("Processed {} collateral data entries", data_len);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid batch data")]
    InvalidBatchData,
} 