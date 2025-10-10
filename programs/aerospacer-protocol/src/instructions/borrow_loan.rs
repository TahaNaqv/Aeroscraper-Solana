use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, MintTo};
use crate::state::*;
use crate::error::*;
use crate::trove_management::*;
use crate::account_management::*;
use crate::oracle::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BorrowLoanParams {
    pub loan_amount: u64,
    pub collateral_denom: String,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: BorrowLoanParams)]
pub struct BorrowLoan<'info> {
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

    // Collateral context accounts
    #[account(
        mut,
        seeds = [b"user_collateral_amount", user.key().as_ref(), params.collateral_denom.as_bytes()],
        bump,
        constraint = user_collateral_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_collateral_amount: Account<'info, UserCollateralAmount>,

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

    // Oracle context - integration with our aerospacer-oracle
    /// CHECK: Our oracle program
    #[account(mut)]
    pub oracle_program: AccountInfo<'info>,
    
    /// CHECK: Oracle state account
    #[account(mut)]
    pub oracle_state: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}



pub fn handler(ctx: Context<BorrowLoan>, params: BorrowLoanParams) -> Result<()> {
    // Validate input parameters
    require!(
        params.loan_amount > 0,
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        params.loan_amount >= MINIMUM_LOAN_AMOUNT,
        AerospacerProtocolError::LoanAmountBelowMinimum
    );
    
    require!(
        !params.collateral_denom.is_empty(),
        AerospacerProtocolError::InvalidAmount
    );
    
    // Check if user has existing trove
    require!(
        ctx.accounts.user_debt_amount.amount > 0,
        AerospacerProtocolError::TroveDoesNotExist
    );
    
    // Create context structs for clean architecture
    let mut trove_ctx = TroveContext {
        user: ctx.accounts.user.clone(),
        user_debt_amount: ctx.accounts.user_debt_amount.clone(),
        liquidity_threshold: ctx.accounts.liquidity_threshold.clone(),
        state: ctx.accounts.state.clone(),
    };
    
    let mut collateral_ctx = CollateralContext {
        user: ctx.accounts.user.clone(),
        user_collateral_amount: ctx.accounts.user_collateral_amount.clone(),
        user_collateral_account: ctx.accounts.user_collateral_account.clone(),
        protocol_collateral_account: ctx.accounts.protocol_collateral_account.clone(),
        total_collateral_amount: ctx.accounts.total_collateral_amount.clone(),
        token_program: ctx.accounts.token_program.clone(),
    };
    
    let mut sorted_ctx = SortedTrovesContext {
        sorted_troves_state: ctx.accounts.sorted_troves_state.clone(),
        state: ctx.accounts.state.clone(),
    };
    
    let oracle_ctx = OracleContext {
        oracle_program: ctx.accounts.oracle_program.clone(),
        oracle_state: ctx.accounts.oracle_state.clone(),
    };
    
    // Use TroveManager for clean implementation
    let result = TroveManager::borrow_loan(
        &mut trove_ctx,
        &mut collateral_ctx,
        &mut sorted_ctx,
        &oracle_ctx,
        params.loan_amount,
    )?;
    
    // Update the actual accounts with the results
    ctx.accounts.user_debt_amount.amount = result.new_debt_amount;
    ctx.accounts.liquidity_threshold.ratio = result.new_icr;
    ctx.accounts.state.total_debt_amount = trove_ctx.state.total_debt_amount;
    ctx.accounts.sorted_troves_state = sorted_ctx.sorted_troves_state;

    // Mint stablecoin
    let mint_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.stable_coin_mint.to_account_info(), // Mint authority
        },
    );
    anchor_spl::token::mint_to(mint_ctx, params.loan_amount)?;

    msg!("Loan borrowed successfully");
    msg!("Amount: {} aUSD", params.loan_amount);
    msg!("Collateral denom: {}", params.collateral_denom);
    msg!("New debt amount: {}", result.new_debt_amount);
    msg!("New ICR: {}", result.new_icr);
    msg!("Collateral amount: {}", result.new_collateral_amount);

    Ok(())
}