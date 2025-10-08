use std::collections::HashMap;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, MintTo};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::trove_helpers::*;
use crate::sorted_troves::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Borrow_loanParams {
    pub loan_amount: u64, // Equivalent to Uint256
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: Borrow_loanParams)]
pub struct Borrow_loan<'info> {
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
        seeds = [b"liquidity_threshold", user.key().as_ref()],
        bump,
        constraint = liquidity_threshold.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub liquidity_threshold: Account<'info, LiquidityThreshold>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    /// CHECK: This is the stable coin mint account
    pub stable_coin_mint: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"sorted_troves_state"],
        bump
    )]
    pub sorted_troves_state: Account<'info, SortedTrovesState>,

    pub token_program: Program<'info, Token>,
}



    pub fn handler(ctx: Context<Borrow_loan>, params: Borrow_loanParams) -> Result<()> {
    let user_debt_amount = &mut ctx.accounts.user_debt_amount;
    let liquidity_threshold = &mut ctx.accounts.liquidity_threshold;
    let state = &mut ctx.accounts.state;

    // Get collateral prices (equivalent to INJECTIVE's query_all_collateral_prices)
    let collateral_prices_response = query_all_collateral_prices(&state)?;
    
    // Convert PriceResponse HashMap to u64 HashMap for trove helpers
    let mut collateral_prices: HashMap<String, u64> = HashMap::new();
    for (denom, price_response) in collateral_prices_response {
        collateral_prices.insert(denom, price_response.price);
    }

    // New total debt amount (equivalent to INJECTIVE's TOTAL_DEBT_AMOUNT.load and checked_add)
    let new_total_debt_amount = safe_add(state.total_debt_amount, params.loan_amount)?;
    state.total_debt_amount = new_total_debt_amount;

    // Update user debt amount (equivalent to INJECTIVE's USER_DEBT_AMOUNT.load and checked_add)
    let new_debt_amount = safe_add(user_debt_amount.amount, params.loan_amount)?;
    user_debt_amount.amount = new_debt_amount;

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

    // Mint stablecoin (equivalent to INJECTIVE's WasmMsg::Execute with Cw20ExecuteMsg::Mint)
    let mint_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.stable_coin_mint.to_account_info(), // Mint authority
        },
    );
    anchor_spl::token::mint_to(mint_ctx, params.loan_amount)?;

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

    msg!("Loan borrowed successfully");
    msg!("Amount: {} aUSD", params.loan_amount);
    msg!("New debt amount: {}", new_debt_amount);
    msg!("Ratio: {}", ratio);

    Ok(())
}