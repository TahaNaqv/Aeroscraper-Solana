use std::collections::HashMap;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::trove_helpers::*;
use crate::sorted_troves::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Remove_collateralParams {
    pub collateral_amount: u64, // Equivalent to Uint256
    pub collateral_denom: String,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: Remove_collateralParams)]
pub struct Remove_collateral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_debt_amount", user.key().as_ref()],
        bump,
        constraint = user_debt_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_debt_amount: Account<'info, UserDebtAmount>,

    #[account(
        mut,
        seeds = [b"user_collateral_amount", user.key().as_ref(), params.collateral_denom.as_bytes()],
        bump,
        constraint = user_collateral_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_collateral_amount: Account<'info, UserCollateralAmount>,

    #[account(
        mut,
        seeds = [b"liquidity_threshold", user.key().as_ref()],
        bump,
        constraint = liquidity_threshold.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub liquidity_threshold: Account<'info, LiquidityThreshold>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,

    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral_amount", params.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_amount: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"sorted_troves_state"],
        bump
    )]
    pub sorted_troves_state: Account<'info, SortedTrovesState>,

    pub token_program: Program<'info, Token>,
}



    pub fn handler(ctx: Context<Remove_collateral>, params: Remove_collateralParams) -> Result<()> {
    let user_debt_amount = &ctx.accounts.user_debt_amount;
    let user_collateral_amount = &mut ctx.accounts.user_collateral_amount;
    let liquidity_threshold = &mut ctx.accounts.liquidity_threshold;
    
    // Store state key before borrowing it mutably
    let state_key = ctx.accounts.state.key();
    let state = &mut ctx.accounts.state;

    // Get collateral prices (equivalent to INJECTIVE's query_all_collateral_prices)
    let collateral_prices_response = query_all_collateral_prices(&state)?;
    
    // Convert PriceResponse HashMap to u64 HashMap for trove helpers
    let mut collateral_prices: HashMap<String, u64> = HashMap::new();
    for (denom, price_response) in collateral_prices_response {
        collateral_prices.insert(denom, price_response.price);
    }

    // Total collateral and debt amount in the protocol (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT.load)
    let old_total_collateral_amount = get_total_collateral_amount(
        &params.collateral_denom,
        &state_key,
        &ctx.remaining_accounts,
    )?;

    // Calculate new total collateral amount (equivalent to INJECTIVE's checked_sub)
    let new_total_collateral_amount = safe_sub(old_total_collateral_amount, params.collateral_amount)?;

    // Update total collateral amount (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT.save)
    update_total_collateral_from_account_info(
        &ctx.accounts.total_collateral_amount,
        -(params.collateral_amount as i64),
    )?;

    // Existing user collateral and debt amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.load)
    let new_collateral_amount = safe_sub(user_collateral_amount.amount, params.collateral_amount)?;

    // New collateral amount for the user (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.save)
    user_collateral_amount.amount = new_collateral_amount;

    // Check the minimum collateral amount (equivalent to INJECTIVE's minimum check)
    require!(
        new_collateral_amount >= MINIMUM_COLLATERAL_AMOUNT,
        AerospacerProtocolError::CollateralBelowMinimum
    );

    // Update liquidity threshold for the user (equivalent to INJECTIVE's get_trove_icr)
        let ratio = get_trove_icr(
            &user_debt_amount,
            &ctx.remaining_accounts, // user_collateral_amount_accounts
            &collateral_prices,
            ctx.accounts.user.key(),
        )?;
    liquidity_threshold.ratio = ratio;

    // Check for the minimum collateral ratio (equivalent to INJECTIVE's check_trove_icr_with_ratio)
    check_trove_icr_with_ratio(state, ratio)?;

    // Transfer collateral from protocol to user (equivalent to INJECTIVE's BankMsg::Send)
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.protocol_collateral_account.to_account_info(),
            to: ctx.accounts.user_collateral_account.to_account_info(),
            authority: ctx.accounts.protocol_collateral_account.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.collateral_amount)?;

    // Reinsert trove into sorted list (equivalent to INJECTIVE's reinsert_trove)
    // TODO: Fix lifetime issues with reinsert_trove
    // let sorted_troves_state = &mut ctx.accounts.sorted_troves_state;
    // reinsert_trove(
    //     sorted_troves_state,
    //     &collateral_prices,
    //     ctx.accounts.user.key(),
    //     ratio,
    //     params.prev_node_id,
    //     params.next_node_id,
    //     &ctx.remaining_accounts,
    // )?;

    msg!("Collateral removed successfully");
    msg!("Removed: {} {}", params.collateral_amount, params.collateral_denom);
    msg!("New collateral amount: {}", new_collateral_amount);
    msg!("Ratio: {}", ratio);

    Ok(())
}