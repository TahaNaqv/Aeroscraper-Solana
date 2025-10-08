use std::collections::HashMap;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, Burn};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::trove_helpers::*;
use crate::sorted_troves::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Repay_loanParams {
    pub amount: u64, // Equivalent to Uint256
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: Repay_loanParams)]
pub struct Repay_loan<'info> {
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
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,

    /// CHECK: This is the stable coin mint account
    pub stable_coin_mint: UncheckedAccount<'info>,

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



pub fn handler<'a>(ctx: Context<'a, 'a, 'a, 'a, Repay_loan<'a>>, params: Repay_loanParams) -> Result<()> {
    let user_debt_amount = &mut ctx.accounts.user_debt_amount;
    let user_collateral_amount = &mut ctx.accounts.user_collateral_amount;
    let liquidity_threshold = &mut ctx.accounts.liquidity_threshold;
    let state = &mut ctx.accounts.state;

    // Get collateral prices (equivalent to INJECTIVE's query_all_collateral_prices)
    let collateral_prices_response = query_all_collateral_prices(&state)?;
    
    // Convert PriceResponse HashMap to u64 HashMap for trove helpers
    let mut collateral_prices: HashMap<String, u64> = HashMap::new();
    for (denom, price_response) in collateral_prices_response {
        collateral_prices.insert(denom, price_response.price);
    }

    // Available collateral denoms for the trove (equivalent to INJECTIVE's get_trove_denoms)
    let trove_denoms = get_trove_denoms(&ctx.remaining_accounts, ctx.accounts.user.key())?;

    // Protocol and user debts (equivalent to INJECTIVE's TOTAL_DEBT_AMOUNT.load and USER_DEBT_AMOUNT.load)
    let old_total_debt_amount = state.total_debt_amount;
    let old_debt_amount = user_debt_amount.amount;

    // New total debt amount (equivalent to INJECTIVE's checked_sub)
    let new_total_debt_amount = safe_sub(old_total_debt_amount, params.amount)?;
    state.total_debt_amount = new_total_debt_amount;

    // Validate repayment amount (equivalent to INJECTIVE's amount > old_debt_amount check)
    require!(
        params.amount <= old_debt_amount,
        AerospacerProtocolError::InvalidAmount
    );

    let ratio: u64;
    let new_debt_amount: u64;

    // Check if this is a full repayment (equivalent to INJECTIVE's old_debt_amount == amount)
    if old_debt_amount == params.amount {
        // If the user is repaying the full debt amount (equivalent to INJECTIVE's true branch)
        new_debt_amount = 0;

        for collateral_denom in trove_denoms {
            // Get total collateral amount (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT.load)
            let total_collateral_amount = get_total_collateral_amount(
                &collateral_denom,
                &ctx.accounts.state.key(),
                &ctx.remaining_accounts,
            )?;
            let collateral_amount = user_collateral_amount.amount;

            let new_total_collateral_amount = safe_sub(total_collateral_amount, collateral_amount)?;

            // Subtract collateral amount from the total collateral amount (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT.save)
            update_total_collateral_from_account_info(
                &ctx.accounts.total_collateral_amount,
                -(collateral_amount as i64),
            )?;

            // Send collateral back to the user (equivalent to INJECTIVE's BankMsg::Send)
            let transfer_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.protocol_collateral_account.to_account_info(),
                    to: ctx.accounts.user_collateral_account.to_account_info(),
                    authority: ctx.accounts.protocol_collateral_account.to_account_info(),
                },
            );
            anchor_spl::token::transfer(transfer_ctx, collateral_amount)?;

            // Remove user collateral amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.remove)
            user_collateral_amount.amount = 0;
        }

        // Remove liquidity threshold and user debt (equivalent to INJECTIVE's LIQUIDITY_THRESHOLD.remove and USER_DEBT_AMOUNT.remove)
        liquidity_threshold.ratio = 0;
        user_debt_amount.amount = 0;

        ratio = 0;

        // Remove trove from sorted list (equivalent to INJECTIVE's remove_trove)
        let sorted_troves_state = &mut ctx.accounts.sorted_troves_state;
        remove_trove(sorted_troves_state, ctx.accounts.user.key(), &ctx.remaining_accounts)?;

        msg!("Trove fully repaid and closed");
    } else {
        // If the user is repaying part of the debt amount (equivalent to INJECTIVE's false branch)
        new_debt_amount = safe_sub(old_debt_amount, params.amount)?;
        user_debt_amount.amount = new_debt_amount;

        // Update liquidity threshold for the user (equivalent to INJECTIVE's get_trove_icr)
        ratio = get_trove_icr(
            user_debt_amount,
            &ctx.remaining_accounts, // user_collateral_amount_accounts
            &collateral_prices,
            ctx.accounts.user.key(),
        )?;
        liquidity_threshold.ratio = ratio;

        // Check for the minimum collateral ratio (equivalent to INJECTIVE's check_trove_icr_with_ratio)
        check_trove_icr_with_ratio(state, ratio)?;

        // Reinsert trove into sorted list (equivalent to INJECTIVE's reinsert_trove)
        let sorted_troves_state = &mut ctx.accounts.sorted_troves_state;
          reinsert_trove(
            sorted_troves_state,
            &collateral_prices,
            ctx.accounts.user.key(),
            ratio,
            params.prev_node_id,
            params.next_node_id,
            &ctx.remaining_accounts,
        )?;

        msg!("Partial repayment successful");
    }

    // Burn stablecoin (equivalent to INJECTIVE's WasmMsg::Execute with Cw20ExecuteMsg::Burn)
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            from: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::burn(burn_ctx, params.amount)?;

    msg!("Loan repaid successfully");
    msg!("Amount: {} aUSD", params.amount);
    msg!("New debt amount: {}", new_debt_amount);
    msg!("Ratio: {}", ratio);

    Ok(())
}