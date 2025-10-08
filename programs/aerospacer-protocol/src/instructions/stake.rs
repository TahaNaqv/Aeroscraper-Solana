use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use crate::state::*;
use crate::utils::*;
use crate::error::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeParams {
    pub amount: u64, // Equivalent to Uint256
}

#[derive(Accounts)]
#[instruction(params: StakeParams)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserStakeAmount::LEN,
        seeds = [b"user_stake_amount", user.key().as_ref()],
        bump
    )]
    pub user_stake_amount: Account<'info, UserStakeAmount>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}



pub fn handler(ctx: Context<Stake>, params: StakeParams) -> Result<()> {
    let user_stake_amount = &mut ctx.accounts.user_stake_amount;
    // Store state key before borrowing it mutably
    let state_key = ctx.accounts.state.key();
    let state = &mut ctx.accounts.state;

    // Update user stake amount
    user_stake_amount.owner = ctx.accounts.user.key();
    user_stake_amount.amount = safe_add(user_stake_amount.amount, params.amount)?;
    user_stake_amount.block_height = Clock::get()?.slot;

    // Update state
    state.total_stake_amount = safe_add(state.total_stake_amount, params.amount)?;

    msg!("Staked successfully");
    msg!("Amount: {} aUSD", params.amount);
    msg!("Total staked: {} aUSD", user_stake_amount.amount);

    Ok(())
}