use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::utils::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Withdraw_liquidation_gainsParams {
    pub collateral_denom: String,
}

#[derive(Accounts)]
#[instruction(params: Withdraw_liquidation_gainsParams)]
pub struct Withdraw_liquidation_gains<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub state: Account<'info, StateAccount>,
    
    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Withdraw_liquidation_gains>, params: Withdraw_liquidation_gainsParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // For now, implement a simplified liquidation gains withdrawal
    // In a full implementation, you would:
    // 1. Calculate user's liquidation gains based on their stake percentage
    // 2. Check if user has unclaimed gains for the specified collateral
    // 3. Transfer collateral gains to user
    // 4. Mark gains as claimed
    
    msg!("Liquidation gains withdrawal started");
    msg!("Collateral denom: {}", params.collateral_denom);
    
    // TODO: Implement full liquidation gains logic
    // 1. Query user's liquidation gains from state
    // 2. Calculate gains based on stake percentage and liquidation events
    // 3. Validate user has unclaimed gains
    // 4. Transfer collateral to user
    // 5. Mark gains as claimed
    // 6. Update liquidation gain tracking
    
    // Placeholder implementation - will be expanded
    msg!("Liquidation gains withdrawal completed successfully");
    msg!("Processed collateral: {}", params.collateral_denom);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("No liquidation gains found")]
    NoLiquidationGains,
    #[msg("Invalid collateral denom")]
    InvalidCollateralDenom,
    #[msg("Gains already claimed")]
    GainsAlreadyClaimed,
    #[msg("Withdrawal failed")]
    WithdrawalFailed,
    #[msg("Overflow occurred")]
    Overflow,
}
