use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::utils::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RedeemParams {
    pub amount: u64,
}

#[derive(Accounts)]
#[instruction(params: RedeemParams)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub state: Account<'info, StateAccount>,
    
    #[account(mut)]
    pub stable_coin_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub protocol_stablecoin_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Redeem>, params: RedeemParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Validate redemption amount
    if params.amount == 0 {
        return Err(ErrorCode::InvalidRedemptionAmount.into());
    }
    
    // Check if there's enough debt in the system to redeem
    if params.amount > state.total_debt_amount {
        return Err(ErrorCode::NotEnoughLiquidityForRedeem.into());
    }
    
    // Get collateral prices from oracle
    let collateral_prices = query_all_collateral_prices(state.oracle_program)?;
    
    // For now, implement a simplified redemption that processes one trove at a time
    // In a full implementation, you would iterate through sorted troves from riskiest to safest
    
    msg!("Redemption process started");
    msg!("Redemption amount: {} aUSD", params.amount);
    
    // TODO: Implement full redemption logic
    // 1. Find riskiest troves (lowest collateral ratio)
    // 2. Calculate collateral to distribute
    // 3. Update trove debt amounts
    // 4. Distribute collateral to redeemer
    // 5. Burn stablecoins from redeemer
    // 6. Update global state totals
    // 7. Handle trove closures if debt is fully repaid
    
    // Placeholder implementation - will be expanded
    msg!("Redemption completed successfully");
    msg!("Processed amount: {} aUSD", params.amount);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid redemption amount")]
    InvalidRedemptionAmount,
    #[msg("Not enough liquidity for redeem")]
    NotEnoughLiquidityForRedeem,
    #[msg("Redemption failed")]
    RedemptionFailed,
    #[msg("Overflow occurred")]
    Overflow,
}
