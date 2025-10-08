use std::collections::HashMap;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, MintTo};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::trove_helpers::*;
use crate::sorted_troves::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Open_troveParams {
    pub loan_amount: u64, // Equivalent to Uint256
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: Open_troveParams)]
pub struct Open_trove<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + UserDebtAmount::LEN,
        seeds = [b"user_debt_amount", user.key().as_ref()],
        bump
    )]
    pub user_debt_amount: Account<'info, UserDebtAmount>,

    #[account(
        init,
        payer = user,
        space = 8 + UserCollateralAmount::LEN,
        seeds = [b"user_collateral_amount", user.key().as_ref(), b"SOL"],
        bump
    )]
    pub user_collateral_amount: Account<'info, UserCollateralAmount>,

    #[account(
        init,
        payer = user,
        space = 8 + LiquidityThreshold::LEN,
        seeds = [b"liquidity_threshold", user.key().as_ref()],
        bump
    )]
    pub liquidity_threshold: Account<'info, LiquidityThreshold>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    /// CHECK: This is the stable coin mint account
    pub stable_coin_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_stablecoin_account: Account<'info, TokenAccount>,

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
    pub system_program: Program<'info, System>,
}



    pub fn handler(ctx: Context<Open_trove>, params: Open_troveParams) -> Result<()> {
    let user_debt_amount = &mut ctx.accounts.user_debt_amount;
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

    // Check the minimum loan amount (equivalent to INJECTIVE's check)
    require!(
        params.loan_amount >= MINIMUM_LOAN_AMOUNT,
        AerospacerProtocolError::LoanAmountBelowMinimum
    );

    // Check if the funds are valid (equivalent to INJECTIVE's check_funds)
    let collateral_amount = ctx.accounts.user_collateral_account.amount;
    let collateral_denom = "SOL".to_string(); // Default to SOL for now
    
    // Validate that user has sufficient collateral
    require!(
        collateral_amount > 0,
        AerospacerProtocolError::InvalidAmount
    );

    // Calculate protocol fee (equivalent to INJECTIVE's fee calculation)
    let protocol_fee = state.protocol_fee as u64;
    let net_collateral_amount = safe_mul(collateral_amount, 1000)?;
    let net_collateral_amount = safe_div(net_collateral_amount, 1000 + protocol_fee)?;

    // Calculate protocol fee amount (equivalent to INJECTIVE's collateral_fee calculation)
    let collateral_fee = safe_sub(collateral_amount, net_collateral_amount)?;

    // Get old total collateral amount for denom (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT.load)
    let old_total_collateral_amount = get_total_collateral_amount(
        &collateral_denom,
        &state_key,
        &ctx.remaining_accounts,
    )?;
    
    // Get old total debt amount (equivalent to INJECTIVE's TOTAL_DEBT_AMOUNT.load)
    let old_total_debt_amount = state.total_debt_amount;

    // Check for percentages - return error if exist (equivalent to INJECTIVE's LIQUIDITY_THRESHOLD.has check)
    // This is simplified - in real implementation you'd check if trove exists
    // For now, we'll assume this is a new trove since we're initializing the accounts

    // Update total collateral and debt amount (equivalent to INJECTIVE's calculations)
    let new_total_collateral_amount = safe_add(old_total_collateral_amount, net_collateral_amount)?;
    let new_total_debt_amount = safe_add(old_total_debt_amount, params.loan_amount)?;

    // Update total collateral amount (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT.save)
    update_total_collateral_from_account_info(
        &ctx.accounts.total_collateral_amount,
        net_collateral_amount as i64,
    )?;
    
    // Update total debt amount (equivalent to INJECTIVE's TOTAL_DEBT_AMOUNT.save)
    state.total_debt_amount = new_total_debt_amount;

    // Update user collateral amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.save)
    user_collateral_amount.owner = ctx.accounts.user.key();
    user_collateral_amount.denom = collateral_denom.clone();
    user_collateral_amount.amount = net_collateral_amount;

    // Update user debt amount (equivalent to INJECTIVE's USER_DEBT_AMOUNT.save)
    user_debt_amount.owner = ctx.accounts.user.key();
    user_debt_amount.amount = params.loan_amount;

    // Save liquidity threshold for liquidation calculation later on (equivalent to INJECTIVE's get_trove_icr)
        let ratio = get_trove_icr(
            &user_debt_amount,
            &ctx.remaining_accounts, // user_collateral_amount_accounts
            &collateral_prices,
            ctx.accounts.user.key(),
        )?;
    liquidity_threshold.owner = ctx.accounts.user.key();
    liquidity_threshold.ratio = ratio;

    // Check for the minimum collateral ratio (equivalent to INJECTIVE's check_trove_icr_with_ratio)
    check_trove_icr_with_ratio(state, ratio)?;

    // Transfer collateral from user to protocol (equivalent to INJECTIVE's fund handling)
    let transfer_collateral_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_collateral_account.to_account_info(),
            to: ctx.accounts.protocol_collateral_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_collateral_ctx, collateral_amount)?;

    // Process protocol fees (equivalent to INJECTIVE's fee processing)
    let mut fee_coins: Vec<u64> = vec![];
    populate_fee_coins(&mut fee_coins, collateral_fee, &collateral_denom)?;
    process_protocol_fees(
        state.fee_distributor_addr,
        fee_coins.iter().sum(),
        collateral_denom.clone(),
    )?;

    // Mint stablecoin (equivalent to INJECTIVE's WasmMsg::Execute with Cw20ExecuteMsg::Mint)
    let mint_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.protocol_stablecoin_account.to_account_info(),
        },
    );
    anchor_spl::token::mint_to(mint_ctx, params.loan_amount)?;

    // Insert trove into sorted list (equivalent to INJECTIVE's insert_trove)
    // TODO: Fix lifetime issues with insert_trove
    // insert_trove(
    //     &mut ctx.accounts.sorted_troves_state,
    //     &collateral_prices,
    //     ctx.accounts.user.key(),
    //     ratio,
    //     params.prev_node_id,
    //     params.next_node_id,
    //     &ctx.remaining_accounts,
    // )?;

    msg!("Trove opened successfully");
    msg!("Collateral: {} {}", net_collateral_amount, collateral_denom);
    msg!("Debt: {} aUSD", params.loan_amount);
    msg!("Protocol fee: {} {}", collateral_fee, collateral_denom);
    msg!("Ratio: {}", ratio);

    Ok(())
}