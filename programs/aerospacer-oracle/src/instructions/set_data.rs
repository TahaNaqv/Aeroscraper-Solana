use anchor_lang::prelude::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetDataParams {
    pub denom: String,
    pub decimal: u8,
    pub price_id: String,
}

#[derive(Accounts)]
#[instruction(params: SetDataParams)]
pub struct SetData<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = state.admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub state: Account<'info, OracleStateAccount>,
}

pub fn handler(ctx: Context<SetData>, params: SetDataParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Create new collateral data
    let collateral_data = CollateralData {
        denom: params.denom.clone(),
        decimal: params.decimal,
        price_id: params.price_id.clone(),
    };
    
    // Check if denom already exists and update, otherwise add new
    if let Some(index) = state.collateral_data.iter().position(|d| d.denom == params.denom) {
        state.collateral_data[index] = collateral_data;
        msg!("Updated collateral data for: {}", params.denom);
    } else {
        state.collateral_data.push(collateral_data);
        msg!("Added new collateral data for: {}", params.denom);
    }
    
    msg!("Set data successful");
    msg!("Denom: {}", params.denom);
    msg!("Decimal: {}", params.decimal);
    msg!("Price ID: {}", params.price_id);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid data")]
    InvalidData,
} 