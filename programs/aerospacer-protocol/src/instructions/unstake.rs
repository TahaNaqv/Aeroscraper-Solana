use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use aerospacer_utils::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UnstakeParams {
    pub amount: u64,
}

#[derive(Accounts)]
#[instruction(params: UnstakeParams)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump,
        constraint = stake.owner == user.key() @ ErrorCode::Unauthorized
    )]
    pub stake: Account<'info, StakeAccount>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_stablecoin_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Unstake>, params: UnstakeParams) -> Result<()> {
    let stake = &mut ctx.accounts.stake;
    let state = &mut ctx.accounts.state;

    // Validate unstake amount
    if params.amount > stake.amount {
        return Err(ErrorCode::InsufficientStake.into());
    }

    if params.amount == 0 {
        return Err(ErrorCode::InvalidUnstakeAmount.into());
    }

    // Update state total stake using safe math
    state.total_stake_amount = safe_sub(state.total_stake_amount, params.amount)?;

    // Update user stake using safe math
    let new_stake_amount = safe_sub(stake.amount, params.amount)?;
    stake.amount = new_stake_amount;
    stake.total_stake_at_time = state.total_stake_amount;
    stake.block_height = Clock::get()?.slot;

    // Calculate new stake percentage using Utils
    if state.total_stake_amount > 0 {
        stake.percentage = calculate_stake_percentage(
            state.total_stake_amount,
            stake.amount,
        )?;
    } else {
        stake.percentage = 0;
    }

    // Transfer stablecoins from protocol to user
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: ctx.accounts.protocol_stablecoin_account.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.amount)?;

    msg!("Unstake successful");
    msg!("Unstaked: {} aUSD", params.amount);
    msg!("Remaining stake: {} aUSD", new_stake_amount);
    msg!("Stake percentage: {}%", stake.percentage / 100);

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Insufficient stake")]
    InsufficientStake,
    #[msg("Invalid unstake amount")]
    InvalidUnstakeAmount,
    #[msg("Overflow occurred")]
    Overflow,
}
