use std::collections::HashMap;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::trove_helpers::*;
use crate::sorted_troves::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Add_collateralParams {
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: Add_collateralParams)]
pub struct Add_collateral<'info> {
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
        seeds = [b"user_collateral_amount", user.key().as_ref(), b"SOL"], // Default to SOL
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
        seeds = [b"total_collateral_amount", b"SOL"],
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



    pub fn handler(ctx: Context<Add_collateral>, params: Add_collateralParams) -> Result<()> {
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

    // Check if the funds are valid (equivalent to INJECTIVE's check_funds)
    let collateral_amount = ctx.accounts.user_collateral_account.amount;
    let collateral_denom = "SOL".to_string(); // Default to SOL for now

    // Calculate protocol fee (equivalent to INJECTIVE's fee calculation)
    let protocol_fee = state.protocol_fee as u64;
    let net_collateral_amount = safe_mul(collateral_amount, 1000)?;
    let net_collateral_amount = safe_div(net_collateral_amount, 1000 + protocol_fee)?;

    // Protocol fee for adding collateral (equivalent to INJECTIVE's collateral_fee calculation)
    let collateral_fee = safe_sub(collateral_amount, net_collateral_amount)?;

    // Fee processing (equivalent to INJECTIVE's fee processing)
    process_protocol_fees(
        state.fee_distributor_addr,
        collateral_fee,
        collateral_denom.clone(),
    )?;

    // Get old total collateral amount (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT.may_load)
    let old_total_collateral_amount = get_total_collateral_amount(
        &collateral_denom,
        &state_key,
        &ctx.remaining_accounts,
    )?;

    // Calculate new total collateral amount (equivalent to INJECTIVE's checked_add)
    let new_total_collateral_amount = safe_add(old_total_collateral_amount, net_collateral_amount)?;

    // Update total collateral amount (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT.save)
    update_total_collateral_from_account_info(
        &ctx.accounts.total_collateral_amount,
        net_collateral_amount as i64,
    )?;

    // Update user collateral amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.save)
    let new_collateral_amount = safe_add(user_collateral_amount.amount, net_collateral_amount)?;
    user_collateral_amount.amount = new_collateral_amount;

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

    // Transfer collateral from user to protocol
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_collateral_account.to_account_info(),
            to: ctx.accounts.protocol_collateral_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, collateral_amount)?;

    msg!("Collateral added successfully");
    msg!("Added: {} {}", net_collateral_amount, collateral_denom);
    msg!("Protocol fee: {} {}", collateral_fee, collateral_denom);
    msg!("New collateral amount: {}", new_collateral_amount);
    msg!("Ratio: {}", ratio);

    Ok(())
}