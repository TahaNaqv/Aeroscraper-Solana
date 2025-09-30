use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::*;
use crate::utils::*;
use crate::error::AerospacerProtocolError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Withdraw_liquidation_gainsParams {
    pub collateral_denom: String,
    pub amount: u64,
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
    
    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral", params.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_by_denom: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Withdraw_liquidation_gains>, params: Withdraw_liquidation_gainsParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Validate parameters
    if params.amount == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }
    
    // In real implementation, you would:
    // 1. Calculate user's liquidation gains based on their stake percentage
    // 2. Check if user has unclaimed gains for the specified collateral
    // 3. Validate the withdrawal amount doesn't exceed available gains
    // 4. Transfer collateral gains to user
    // 5. Mark gains as claimed
    // 6. Update liquidation gain tracking state
    
    // For now, implement simplified liquidation gains withdrawal
    let user_stake_percentage = if state.total_stake_amount > 0 {
        // In real implementation, you'd look up the user's stake
        // For now, use a placeholder percentage
        1000 // 10% as placeholder
    } else {
        0
    };
    
    if user_stake_percentage == 0 {
        return Err(ErrorCode::NoLiquidationGains.into());
    }
    
        // Calculate available liquidation gains (simplified)
        let available_gains = if state.collateral_denoms.contains(&params.collateral_denom) {
            let total_collateral = if !ctx.accounts.total_collateral_by_denom.data_is_empty() {
                let data = ctx.accounts.total_collateral_by_denom.try_borrow_data()?;
                let total_collateral: TotalCollateralByDenom = 
                    TotalCollateralByDenom::try_deserialize(&mut data.as_ref())?;
                total_collateral.total_amount
            } else {
                0
            };
            // In real implementation, you'd calculate based on actual liquidation events
            safe_mul(total_collateral, user_stake_percentage)? / 10000
        } else {
            return Err(AerospacerProtocolError::InvalidCollateralDenom.into());
        };
    
    if params.amount > available_gains {
        return Err(ErrorCode::InsufficientGains.into());
    }
    
    // Transfer collateral to user
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.protocol_collateral_account.to_account_info(),
            to: ctx.accounts.user_collateral_account.to_account_info(),
            authority: ctx.accounts.protocol_collateral_account.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.amount)?;
    
        // Update per-denom collateral total PDA
        if !ctx.accounts.total_collateral_by_denom.data_is_empty() {
            let mut data = ctx.accounts.total_collateral_by_denom.try_borrow_mut_data()?;
            let mut total_collateral: TotalCollateralByDenom = 
                TotalCollateralByDenom::try_deserialize(&mut data.as_ref())?;
            total_collateral.total_amount = safe_sub(
                total_collateral.total_amount,
                params.amount,
            )?;
            total_collateral.last_updated = Clock::get()?.unix_timestamp;
            
            total_collateral.try_serialize(&mut *data)?;
        }
    
    msg!("Liquidation gains withdrawal completed successfully");
    msg!("Collateral denom: {}", params.collateral_denom);
    msg!("Amount withdrawn: {}", params.amount);
    msg!("User stake percentage: {}%", user_stake_percentage / 100);
    
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
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient gains")]
    InsufficientGains,
}
