use anchor_lang::prelude::*;
use anchor_spl::token::{Token, transfer, Transfer};
use std::str::FromStr;
use crate::state::{FeeStateAccount, FEE_ADDR_1, FEE_ADDR_2};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DistributeFeeParams {
    pub fee_amount: u64, // Equivalent to INJECTIVE's info.funds amount
}

#[derive(Accounts)]
#[instruction(params: DistributeFeeParams)]
pub struct DistributeFee<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub state: Account<'info, FeeStateAccount>,
    
    /// CHECK: This is the payer's token account (source of fees)
    #[account(mut)]
    pub payer_token_account: UncheckedAccount<'info>,
    
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

pub fn handler(ctx: Context<DistributeFee>, params: DistributeFeeParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Use the fee amount from parameters (equivalent to INJECTIVE's info.funds)
    let fee_amount = params.fee_amount;
    
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
        // Distribute to stability pool (equivalent to INJECTIVE's stake contract)
        msg!("Distributing fees to stability pool");
        
        // Transfer all fees to stability pool
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer_token_account.to_account_info(),
                to: ctx.accounts.stability_pool_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        
        transfer(transfer_ctx, fee_amount)?;
        
        msg!("Fees distributed to stability pool successfully: {}", fee_amount);
    } else {
        // Distribute to fee addresses (50/50 split - equivalent to INJECTIVE's FEE_ADDR_1 and FEE_ADDR_2)
        let half_amount = fee_amount / 2;
        let remaining_amount = fee_amount - half_amount; // Handle odd amounts
        
        msg!("Distributing fees to fee addresses (50/50 split)");
        msg!("Half amount: {}", half_amount);
        msg!("Remaining amount: {}", remaining_amount);
        
        // Transfer half to fee address 1 (using hardcoded constant like INJECTIVE)
        if half_amount > 0 {
            let fee_address_1_pubkey = Pubkey::from_str(FEE_ADDR_1).unwrap();
            let transfer_ctx_1 = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.fee_address_1_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            );
            
            transfer(transfer_ctx_1, half_amount)?;
            msg!("Fees transferred to fee address 1: {}", half_amount);
        }
        
        // Transfer remaining to fee address 2 (using hardcoded constant like INJECTIVE)
        if remaining_amount > 0 {
            let fee_address_2_pubkey = Pubkey::from_str(FEE_ADDR_2).unwrap();
            let transfer_ctx_2 = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.fee_address_2_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            );
            
            transfer(transfer_ctx_2, remaining_amount)?;
            msg!("Fees transferred to fee address 2: {}", remaining_amount);
        }
        
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
