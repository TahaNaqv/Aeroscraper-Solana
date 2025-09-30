use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Burn, Transfer};
use crate::state::*;
use crate::utils::*;
use crate::sorted_troves::*;
use crate::trove_helpers::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RedeemParams {
    pub amount: u64,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
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
    
    /// CHECK: Per-denom collateral total PDA for SOL
    #[account(
        mut,
        seeds = [b"total_collateral", b"SOL"],
        bump
    )]
    pub total_collateral_by_denom: AccountInfo<'info>,
    
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
    let collateral_prices = query_all_collateral_prices(state.oracle_program, &state.collateral_denoms)?;
    
    let mut total_collateral_to_send: u64 = 0;
    let mut total_debt_to_burn: u64 = 0;
    
    // In real implementation, you would:
    // 1. Find riskiest troves (lowest collateral ratio) from sorted list
    // 2. Calculate collateral to distribute based on debt ratio
    // 3. Update trove debt amounts
    // 4. Distribute collateral to redeemer
    // 5. Burn stablecoins from redeemer
    // 6. Update global state totals
    // 7. Handle trove closures if debt is fully repaid
    
    // For now, implement simplified redemption logic
    let redemption_ratio = safe_div(params.amount, state.total_debt_amount)?;
    
        // Calculate collateral to send (simplified)
        if state.collateral_denoms.contains(&"SOL".to_string()) {
            // Get collateral amount from PDA
            let collateral_amount = if !ctx.accounts.total_collateral_by_denom.data_is_empty() {
                let data = ctx.accounts.total_collateral_by_denom.try_borrow_data()?;
                let total_collateral: TotalCollateralByDenom = 
                    TotalCollateralByDenom::try_deserialize(&mut data.as_ref())?;
                total_collateral.total_amount
            } else {
                0
            };
            
            total_collateral_to_send = safe_mul(collateral_amount, redemption_ratio)?;
            
            // Update per-denom collateral total PDA
            if !ctx.accounts.total_collateral_by_denom.data_is_empty() {
                let mut data = ctx.accounts.total_collateral_by_denom.try_borrow_mut_data()?;
                let mut total_collateral: TotalCollateralByDenom = 
                    TotalCollateralByDenom::try_deserialize(&mut data.as_ref())?;
                total_collateral.total_amount = safe_sub(
                    total_collateral.total_amount,
                    total_collateral_to_send,
                )?;
                total_collateral.last_updated = Clock::get()?.unix_timestamp;
                
                total_collateral.try_serialize(&mut *data)?;
            }
        }
    
    total_debt_to_burn = params.amount;
    
    // Process protocol fees
    let (protocol_fee, net_collateral) = calculate_redemption_fees(
        total_collateral_to_send,
        state.protocol_fee,
    )?;
    
    // Process protocol fees
    process_protocol_fees(
        state.fee_distributor,
        protocol_fee,
        "SOL".to_string(),
    )?;
    
    // Update state total debt
    state.total_debt_amount = safe_sub(state.total_debt_amount, total_debt_to_burn)?;
    
    // Transfer collateral to user
    let transfer_collateral_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.protocol_collateral_account.to_account_info(),
            to: ctx.accounts.user_collateral_account.to_account_info(),
            authority: ctx.accounts.protocol_collateral_account.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_collateral_ctx, net_collateral)?;
    
    // Burn stablecoins from user
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            from: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::burn(burn_ctx, total_debt_to_burn)?;
    
    msg!("Redemption process completed successfully");
    msg!("Redemption amount: {} aUSD", params.amount);
    msg!("Collateral sent: {} SOL", net_collateral);
    msg!("Protocol fee: {} SOL", protocol_fee);
    msg!("Debt burned: {} aUSD", total_debt_to_burn);
    
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
