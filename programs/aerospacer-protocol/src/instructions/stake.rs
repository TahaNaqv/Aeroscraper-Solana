use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::msg::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeParams {
    pub amount: u64,
}

#[derive(Accounts)]
#[instruction(params: StakeParams)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + StakeAccount::LEN,
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub stake: Account<'info, StakeAccount>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_stablecoin_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Stake>, params: StakeParams) -> Result<()> {
    let stake = &mut ctx.accounts.stake;
    let state = &mut ctx.accounts.state;

    // Validate stake amount using Utils
    if params.amount < crate::state::MINIMUM_LOAN_AMOUNT / 1000 {
        return Err(ErrorCode::StakeAmountTooSmall.into());
    }

    // Update state total stake using safe math
    state.total_stake_amount = safe_add(state.total_stake_amount, params.amount)?;

    // Update user stake using safe math
    let new_stake_amount = safe_add(stake.amount, params.amount)?;
    stake.owner = ctx.accounts.user.key();
    stake.amount = new_stake_amount;
    stake.total_stake_at_time = state.total_stake_amount;
    stake.block_height = Clock::get()?.slot;

    // Calculate stake percentage using Utils
    stake.percentage = calculate_stake_percentage(
        state.total_stake_amount,
        stake.amount,
    )?;

    // Transfer stablecoins from user to protocol
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: ctx.accounts.user_stablecoin_account.to_account_info(),
            to: ctx.accounts.protocol_stablecoin_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.amount)?;

    msg!("Stake successful");
    msg!("Staked: {} aUSD", params.amount);
    msg!("Total stake: {} aUSD", new_stake_amount);
    msg!("Stake percentage: {}%", stake.percentage / 100);

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Stake amount too small")]
    StakeAmountTooSmall,
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Invalid stake parameters")]
    InvalidStakeParameters,
}
