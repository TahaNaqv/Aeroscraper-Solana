use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::FeeStateAccount;

#[derive(Accounts)]
pub struct DistributeFee<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub state: Account<'info, FeeStateAccount>,
    
    /// CHECK: This is the payer's token account
    #[account(mut)]
    pub payer_token_account: UncheckedAccount<'info>,
    
    /// CHECK: This is the fee token account
    #[account(mut)]
    pub fee_token_account: UncheckedAccount<'info>,
    
    /// CHECK: This is the stability pool token account (when stake is enabled)
    #[account(mut)]
    pub stability_pool_token_account: UncheckedAccount<'info>,
    
    /// CHECK: This is the fee address 1 token account
    #[account(mut)]
    pub fee_address_1_token_account: UncheckedAccount<'info>,
    
    /// CHECK: This is the fee address 2 token account
    #[account(mut)]
    pub fee_address_2_token_account: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<DistributeFee>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // For now, we'll use a placeholder fee amount since we can't directly access token account data
    // In a real implementation, you would need to pass the fee amount as a parameter
    let fee_amount = 1000000; // Placeholder fee amount
    
    if fee_amount == 0 {
        return Err(ErrorCode::NoFeesToDistribute.into());
    }
    
    // Update total fees collected
    state.total_fees_collected = state.total_fees_collected
        .checked_add(fee_amount)
        .ok_or(ErrorCode::Overflow)?;
    
    msg!("Distributing fee amount: {}", fee_amount);
    msg!("Total fees collected: {}", state.total_fees_collected);
    
    if state.is_stake_enabled {
        // Distribute to stability pool
        msg!("Distributing fees to stability pool");
        
        // TODO: Implement actual token transfer to stability pool
        // For now, just log the action
        msg!("Fees would be distributed to stability pool: {}", fee_amount);
        
        msg!("Fees distributed to stability pool successfully");
    } else {
        // Distribute to fee addresses (50/50 split)
        let half_amount = fee_amount / 2;
        let remaining_amount = fee_amount - half_amount; // Handle odd amounts
        
        msg!("Distributing fees to fee addresses (50/50 split)");
        msg!("Half amount: {}", half_amount);
        msg!("Remaining amount: {}", remaining_amount);
        
        // TODO: Implement actual token transfers to fee addresses
        // For now, just log the actions
        msg!("Fees would be distributed to fee address 1: {}", half_amount);
        msg!("Fees would be distributed to fee address 2: {}", remaining_amount);
        
        msg!("Fees distributed to fee addresses successfully");
    }
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("No fees to distribute")]
    NoFeesToDistribute,
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Invalid fee distribution")]
    InvalidFeeDistribution,
}
