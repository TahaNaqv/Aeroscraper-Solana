use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, Burn};
use crate::state::*;
use crate::error::*;
use crate::trove_management::*;
use crate::account_management::*;
use crate::oracle::*;
use crate::sorted_troves;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RepayLoanParams {
    pub amount: u64,
    pub collateral_denom: String,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: RepayLoanParams)]
pub struct RepayLoan<'info> {
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
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"protocol_collateral_vault", params.collateral_denom.as_bytes()],
        bump
    )]
    pub protocol_collateral_account: Account<'info, TokenAccount>,

    /// CHECK: This is the stable coin mint account
    pub stable_coin_mint: UncheckedAccount<'info>,

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

    // Node account for sorted troves linked list (manually closed if full repayment)
    #[account(
        mut,
        seeds = [b"node", user.key().as_ref()],
        bump
    )]
    pub node: Account<'info, Node>,

    // Oracle context - integration with our aerospacer-oracle
    /// CHECK: Our oracle program - validated against state
    #[account(
        mut,
        constraint = oracle_program.key() == state.oracle_helper_addr @ AerospacerProtocolError::Unauthorized
    )]
    pub oracle_program: AccountInfo<'info>,
    
    /// CHECK: Oracle state account - validated against state
    #[account(
        mut,
        constraint = oracle_state.key() == state.oracle_state_addr @ AerospacerProtocolError::Unauthorized
    )]
    pub oracle_state: AccountInfo<'info>,
    
    /// CHECK: Pyth price account for collateral price feed
    pub pyth_price_account: AccountInfo<'info>,
    
    /// Clock sysvar for timestamp validation
    pub clock: Sysvar<'info, Clock>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<RepayLoan>, params: RepayLoanParams) -> Result<()> {
    // Validate input parameters
    require!(
        params.amount > 0,
        AerospacerProtocolError::InvalidAmount
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
    
    // Check if user has sufficient stablecoins
    require!(
        params.amount <= ctx.accounts.user_stablecoin_account.amount,
        AerospacerProtocolError::InsufficientCollateral
    );
    
    // Check if repayment amount doesn't exceed debt
    require!(
        params.amount <= ctx.accounts.user_debt_amount.amount,
        AerospacerProtocolError::InvalidAmount
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
        pyth_price_account: ctx.accounts.pyth_price_account.clone(),
        clock: ctx.accounts.clock.to_account_info(),
    };
    
    // Use TroveManager for clean implementation
    let result = TroveManager::repay_loan(
        &mut trove_ctx,
        &mut collateral_ctx,
        &oracle_ctx,
        params.amount,
    )?;
    
    // Update the actual accounts with the results
    ctx.accounts.user_debt_amount.amount = result.new_debt_amount;
    ctx.accounts.liquidity_threshold.ratio = result.new_icr;
    ctx.accounts.user_collateral_amount.amount = result.new_collateral_amount;
    ctx.accounts.state.total_debt_amount = trove_ctx.state.total_debt_amount;

    // If debt is fully repaid, close the node account and remove from sorted list
    if result.new_debt_amount == 0 {
        // Remove from sorted troves using remaining_accounts for neighbor nodes
        sorted_troves::remove_trove(
            &mut *ctx.accounts.sorted_troves_state,
            ctx.accounts.user.key(),
            ctx.remaining_accounts,
        )?;
        
        // Close the node account manually by zeroing data and transferring lamports
        let node_account_info = ctx.accounts.node.to_account_info();
        let user_account_info = ctx.accounts.user.to_account_info();
        
        // Transfer lamports to user (close account)
        let dest_starting_lamports = user_account_info.lamports();
        **user_account_info.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(node_account_info.lamports())
            .unwrap();
        **node_account_info.lamports.borrow_mut() = 0;
        
        // Zero out account data
        let mut data = node_account_info.try_borrow_mut_data()?;
        data.fill(0);
        
        msg!("Trove fully repaid - Node account closed and removed from sorted list");
    }
    
    // Write sorted_troves_state AFTER potential removal to persist changes
    ctx.accounts.sorted_troves_state = sorted_ctx.sorted_troves_state;

    // Burn stablecoin
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
    msg!("Collateral denom: {}", params.collateral_denom);
    msg!("New debt amount: {}", result.new_debt_amount);
    msg!("New ICR: {}", result.new_icr);
    msg!("Collateral amount: {}", result.new_collateral_amount);

    Ok(())
}