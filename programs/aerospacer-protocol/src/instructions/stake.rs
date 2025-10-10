use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
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
        bump,
        constraint = user_stake_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_stake_amount: Account<'info, UserStakeAmount>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    // Token accounts for staking
    #[account(
        mut,
        constraint = user_stablecoin_account.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    /// CHECK: Protocol stablecoin vault PDA
    #[account(
        mut,
        seeds = [b"protocol_stablecoin_vault"],
        bump
    )]
    pub protocol_stablecoin_vault: AccountInfo<'info>,

    /// CHECK: This is the stable coin mint account
    #[account(
        constraint = stable_coin_mint.key() == state.stable_coin_addr @ AerospacerProtocolError::InvalidMint
    )]
    pub stable_coin_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}



pub fn handler(ctx: Context<Stake>, params: StakeParams) -> Result<()> {
    // Validate input parameters
    require!(
        params.amount > 0,
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        params.amount >= MINIMUM_LOAN_AMOUNT, // Use same minimum as loans
        AerospacerProtocolError::InvalidAmount
    );
    
    // Check if user has sufficient stablecoins
    require!(
        ctx.accounts.user_stablecoin_account.amount >= params.amount,
        AerospacerProtocolError::InsufficientCollateral
    );

    let user_stake_amount = &mut ctx.accounts.user_stake_amount;
    let state = &mut ctx.accounts.state;

    // Transfer stablecoins from user to protocol vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_stablecoin_account.to_account_info(),
            to: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.amount)?;

    // Update user stake amount
    user_stake_amount.owner = ctx.accounts.user.key();
    user_stake_amount.amount = safe_add(user_stake_amount.amount, params.amount)?;
    user_stake_amount.block_height = Clock::get()?.slot;

    // Update state
    state.total_stake_amount = safe_add(state.total_stake_amount, params.amount)?;

    msg!("Staked successfully");
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Amount: {} aUSD", params.amount);
    msg!("Total staked: {} aUSD", user_stake_amount.amount);
    msg!("Total protocol stake: {} aUSD", state.total_stake_amount);

    Ok(())
}