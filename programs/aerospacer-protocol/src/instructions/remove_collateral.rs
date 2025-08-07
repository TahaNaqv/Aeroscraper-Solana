use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::utils::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Remove_collateralParams {
    pub amount: u64,
    pub collateral_denom: String,
}

#[derive(Accounts)]
#[instruction(params: Remove_collateralParams)]
pub struct Remove_collateral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"trove", user.key().as_ref()],
        bump,
        constraint = trove.owner == user.key() @ ErrorCode::Unauthorized,
        constraint = trove.is_active @ ErrorCode::TroveNotActive
    )]
    pub trove: Account<'info, TroveAccount>,
    
    #[account(mut)]
    pub state: Account<'info, StateAccount>,
    
    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Remove_collateral>, params: Remove_collateralParams) -> Result<()> {
    let trove = &mut ctx.accounts.trove;
    let state = &mut ctx.accounts.state;
    
    // Validate collateral denom matches trove
    if trove.collateral_denom != params.collateral_denom {
        return Err(ErrorCode::InvalidCollateralDenom.into());
    }
    
    // Validate amount doesn't exceed trove collateral
    if params.amount > trove.collateral_amount {
        return Err(ErrorCode::InsufficientCollateral.into());
    }
    
    // Get collateral price from oracle
    let collateral_price = query_collateral_price(
        state.oracle_program,
        params.collateral_denom.clone(),
    )?;
    
    // Calculate new collateral amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT update)
    let new_collateral_amount = trove.collateral_amount
        .checked_sub(params.amount)
        .ok_or(ErrorCode::Overflow)?;
    
    // Check minimum collateral amount (equivalent to INJECTIVE's 5 SOL minimum)
    const MINIMUM_COLLATERAL_AMOUNT: u64 = 5_000_000_000; // 5 SOL with 9 decimals
    if new_collateral_amount < MINIMUM_COLLATERAL_AMOUNT {
        return Err(ErrorCode::CollateralBelowMinimum.into());
    }
    
    // Update trove collateral
    trove.collateral_amount = new_collateral_amount;
    trove.collateral_ratio = calculate_collateral_ratio(
        trove.collateral_amount,
        trove.debt_amount,
        collateral_price,
    )?;
    
    // Validate the updated trove meets minimum collateral ratio (equivalent to INJECTIVE's check_trove_icr_with_ratio)
    validate_trove_parameters(
        trove.collateral_amount,
        trove.debt_amount,
        state.minimum_collateral_ratio as u64,
        collateral_price,
    )?;
    
    // Update state totals (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT update)
    if let Some(index) = state.collateral_denoms.iter().position(|d| d == &params.collateral_denom) {
        state.total_collateral_amounts[index] = state.total_collateral_amounts[index]
            .checked_sub(params.amount)
            .ok_or(ErrorCode::Overflow)?;
    } else {
        return Err(ErrorCode::InvalidCollateralDenom.into());
    }
    
    // Transfer collateral from protocol to user (equivalent to INJECTIVE's BankMsg::Send)
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: ctx.accounts.protocol_collateral_account.to_account_info(),
            to: ctx.accounts.user_collateral_account.to_account_info(),
            authority: ctx.accounts.protocol_collateral_account.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.amount)?;
    
    msg!("Collateral removed successfully");
    msg!("Removed: {} {}", params.amount, params.collateral_denom);
    msg!("Remaining collateral: {} {}", new_collateral_amount, params.collateral_denom);
    msg!("New collateral ratio: {}%", trove.collateral_ratio / 100);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Trove not active")]
    TroveNotActive,
    #[msg("Invalid collateral denom")]
    InvalidCollateralDenom,
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
    #[msg("Collateral below minimum")]
    CollateralBelowMinimum,
    #[msg("Overflow occurred")]
    Overflow,
}
