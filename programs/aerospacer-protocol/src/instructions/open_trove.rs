use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, MintTo, Transfer};
use crate::state::*;
use crate::error::*;
use crate::account_management::*;
use crate::oracle::*;
use crate::trove_management::*;
use crate::trove_management::TroveManager;
use crate::state::{DECIMAL_FRACTION_18, MINIMUM_LOAN_AMOUNT, MINIMUM_COLLATERAL_AMOUNT};

// Oracle integration is now handled via our aerospacer-oracle contract

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Open_troveParams {
    pub loan_amount: u64,
    pub collateral_denom: String,
    pub collateral_amount: u64,
}

#[derive(Accounts)]
#[instruction(params: Open_troveParams)]
pub struct Open_trove<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    // Trove context accounts
    #[account(
        init,
        payer = user,
        space = 8 + UserDebtAmount::LEN,
        seeds = [b"user_debt_amount", user.key().as_ref()],
        bump,
        constraint = user_debt_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_debt_amount: Account<'info, UserDebtAmount>,
    
    #[account(
        init,
        payer = user,
        space = 8 + LiquidityThreshold::LEN,
        seeds = [b"liquidity_threshold", user.key().as_ref()],
        bump,
        constraint = liquidity_threshold.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub liquidity_threshold: Account<'info, LiquidityThreshold>,
    
    // Collateral context accounts
    #[account(
        init,
        payer = user,
        space = 8 + UserCollateralAmount::LEN,
        seeds = [b"user_collateral_amount", user.key().as_ref(), params.collateral_denom.as_bytes()],
        bump,
        constraint = user_collateral_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_collateral_amount: Account<'info, UserCollateralAmount>,
    
    #[account(
        mut,
        constraint = user_collateral_account.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
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
    
    // Sorted troves context accounts
    #[account(
        mut,
        seeds = [b"sorted_troves_state"],
        bump
    )]
    pub sorted_troves_state: Account<'info, SortedTrovesState>,
    
    // State account
    #[account(mut)]
    pub state: Account<'info, StateAccount>,
    
    // Token accounts
    #[account(
        mut,
        constraint = user_stablecoin_account.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_stablecoin_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub protocol_stablecoin_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is the stable coin mint account
    #[account(
        constraint = stable_coin_mint.key() == state.stable_coin_addr @ AerospacerProtocolError::InvalidMint
    )]
    pub stable_coin_mint: UncheckedAccount<'info>,
    
    // Oracle context - integration with our aerospacer-oracle
    /// CHECK: Our oracle program
    #[account(mut)]
    pub oracle_program: AccountInfo<'info>,
    
    /// CHECK: Oracle state account
    #[account(mut)]
    pub oracle_state: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Open_trove>, params: Open_troveParams) -> Result<()> {
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
        params.collateral_amount > 0,
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        params.collateral_amount >= MINIMUM_COLLATERAL_AMOUNT,
        AerospacerProtocolError::CollateralBelowMinimum
    );
    
    require!(
        !params.collateral_denom.is_empty(),
        AerospacerProtocolError::InvalidAmount
    );
    
    // Check if user already has a trove (should be 0 for new trove)
    require!(
        ctx.accounts.user_debt_amount.amount == 0,
        AerospacerProtocolError::TroveExists
    );
    
    // Check if user has sufficient collateral
    require!(
        ctx.accounts.user_collateral_account.amount >= params.collateral_amount,
        AerospacerProtocolError::InsufficientCollateral
    );
    
    // Initialize user debt amount
    ctx.accounts.user_debt_amount.owner = ctx.accounts.user.key();
    ctx.accounts.user_debt_amount.amount = 0; // Will be set below
    
    // Initialize user collateral amount
    ctx.accounts.user_collateral_amount.owner = ctx.accounts.user.key();
    ctx.accounts.user_collateral_amount.denom = params.collateral_denom.clone();
    ctx.accounts.user_collateral_amount.amount = 0; // Will be set below
    
    // Initialize liquidity threshold
    ctx.accounts.liquidity_threshold.owner = ctx.accounts.user.key();
    ctx.accounts.liquidity_threshold.ratio = 0; // Will be set below
    
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
    let result = TroveManager::open_trove(
        &mut trove_ctx,
        &mut collateral_ctx,
        &mut sorted_ctx,
        &oracle_ctx,
        params.loan_amount,
        params.collateral_amount,
        params.collateral_denom.clone(),
    )?;
    
    // Update the actual accounts with the results
    ctx.accounts.user_debt_amount.amount = result.new_debt_amount;
    ctx.accounts.liquidity_threshold.ratio = result.new_icr;
    ctx.accounts.user_collateral_amount.amount = result.new_collateral_amount;
    ctx.accounts.state.total_debt_amount = trove_ctx.state.total_debt_amount;
    ctx.accounts.sorted_troves_state = sorted_ctx.sorted_troves_state;
    
    // Mint stablecoins to user
    let mint_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.protocol_stablecoin_account.to_account_info(),
        },
    );
    anchor_spl::token::mint_to(mint_ctx, params.loan_amount)?;
    
    // Log success
    msg!("Trove opened successfully");
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Loan amount: {} aUSD", params.loan_amount);
    msg!("Collateral: {} {}", params.collateral_amount, params.collateral_denom);
    msg!("ICR: {}", result.new_icr);
    
    Ok(())
}