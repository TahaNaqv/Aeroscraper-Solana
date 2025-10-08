use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::*;
use crate::utils::*;
use crate::error::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Withdraw_liquidation_gainsParams {
    pub collateral_denom: String,
}

#[derive(Accounts)]
#[instruction(params: Withdraw_liquidation_gainsParams)]
pub struct Withdraw_liquidation_gains<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserLiquidationCollateralGain::LEN,
        seeds = [b"user_liquidation_collateral_gain", user.key().as_ref()],
        bump
    )]
    pub user_liquidation_collateral_gain: Account<'info, UserLiquidationCollateralGain>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    /// CHECK: Protocol collateral vault PDA
    #[account(
        mut,
        seeds = [b"protocol_collateral_vault", params.collateral_denom.as_bytes()],
        bump
    )]
    pub protocol_collateral_vault: AccountInfo<'info>,

    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral_amount", params.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_amount: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}



pub fn handler(ctx: Context<Withdraw_liquidation_gains>, params: Withdraw_liquidation_gainsParams) -> Result<()> {
    let user_liquidation_collateral_gain = &mut ctx.accounts.user_liquidation_collateral_gain;
    // Store state key before borrowing it mutably
    let state_key = ctx.accounts.state.key();
    let state = &mut ctx.accounts.state;

    // Get liquidation gains for the user
    let gains = get_liquidation_gains(
        ctx.accounts.user.key(),
        &state,
        &[], // user_liquidation_collateral_gain_accounts
        &[], // total_liquidation_collateral_gain_accounts
        &[], // user_stake_amount_accounts
    )?;
    
    // Find the gain for the specific collateral denom
    let mut total_gain = 0u64;
    for gain in gains {
        if gain.denom == params.collateral_denom {
            total_gain = safe_add(total_gain, gain.amount)?;
        }
    }

    // Check if user has any gains to withdraw
    require!(
        total_gain > 0,
        AerospacerProtocolError::CollateralRewardsNotFound
    );

    // Update liquidation gains account
    user_liquidation_collateral_gain.user = ctx.accounts.user.key();
    user_liquidation_collateral_gain.block_height = Clock::get()?.slot;
    user_liquidation_collateral_gain.claimed = true;

    // Transfer collateral from protocol to user
    let transfer_seeds = &[
        b"protocol_collateral_vault".as_ref(),
        params.collateral_denom.as_bytes(),
        &[ctx.bumps.protocol_collateral_vault],
    ];
    let transfer_signer = &[&transfer_seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.protocol_collateral_vault.to_account_info(),
            to: ctx.accounts.user_collateral_account.to_account_info(),
            authority: ctx.accounts.protocol_collateral_vault.to_account_info(),
        },
        transfer_signer,
    );
    anchor_spl::token::transfer(transfer_ctx, total_gain)?;

    // Update per-denom collateral total PDA
    update_total_collateral_from_account_info(
        &ctx.accounts.total_collateral_amount,
        -(total_gain as i64),
    )?;

    msg!("Liquidation gains withdrawn successfully");
    msg!("Amount: {} {}", total_gain, params.collateral_denom);
    msg!("User: {}", ctx.accounts.user.key());

    Ok(())
}