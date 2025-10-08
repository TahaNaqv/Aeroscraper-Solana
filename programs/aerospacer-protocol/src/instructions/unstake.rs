use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::*;
use crate::utils::*;
use crate::error::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UnstakeParams {
    pub amount: u64, // Equivalent to Uint256
}

#[derive(Accounts)]
#[instruction(params: UnstakeParams)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_stake_amount", user.key().as_ref()],
        bump,
        constraint = user_stake_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_stake_amount: Account<'info, UserStakeAmount>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    /// CHECK: Protocol stablecoin vault PDA
    #[account(
        mut,
        seeds = [b"protocol_stablecoin_vault"],
        bump
    )]
    pub protocol_stablecoin_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}



pub fn handler(ctx: Context<Unstake>, params: UnstakeParams) -> Result<()> {
    let user_stake_amount = &mut ctx.accounts.user_stake_amount;
    // Store state key before borrowing it mutably
    let state_key = ctx.accounts.state.key();
    let state = &mut ctx.accounts.state;

    // Check if user has enough stake
    require!(
        user_stake_amount.amount >= params.amount,
        AerospacerProtocolError::InvalidAmount
    );

    // Transfer stablecoin back to user from protocol vault (Injective: CW20 transfer)
    let transfer_seeds = &[
        b"protocol_stablecoin_vault".as_ref(),
        &[ctx.bumps.protocol_stablecoin_vault],
    ];
    let transfer_signer = &[&transfer_seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
        },
        transfer_signer,
    );
    anchor_spl::token::transfer(transfer_ctx, params.amount)?;

    // Update user stake amount
    user_stake_amount.amount = safe_sub(user_stake_amount.amount, params.amount)?;
    user_stake_amount.block_height = Clock::get()?.slot;

    // Update state
    state.total_stake_amount = safe_sub(state.total_stake_amount, params.amount)?;

    msg!("Unstaked successfully");
    msg!("Amount: {} aUSD", params.amount);
    msg!("Remaining stake: {} aUSD", user_stake_amount.amount);

    Ok(())
}