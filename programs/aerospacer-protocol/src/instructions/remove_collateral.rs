use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::*;
use crate::error::*;
use crate::trove_management::*;
use crate::account_management::*;
use crate::oracle::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveCollateralParams {
    pub collateral_amount: u64,
    pub collateral_denom: String,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: RemoveCollateralParams)]
pub struct RemoveCollateral<'info> {
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

    #[account(
        mut,
        seeds = [b"protocol_collateral_vault", params.collateral_denom.as_bytes()],
        bump
    )]
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

    // Node account for sorted troves linked list (for reinsertion with new ICR)
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



pub fn handler(ctx: Context<RemoveCollateral>, params: RemoveCollateralParams) -> Result<()> {
    // Validate input parameters
    require!(
        params.collateral_amount > 0,
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        !params.collateral_denom.is_empty(),
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        params.collateral_amount <= ctx.accounts.user_collateral_amount.amount,
        AerospacerProtocolError::InsufficientCollateral
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
    let result = TroveManager::remove_collateral(
        &mut trove_ctx,
        &mut collateral_ctx,
        &oracle_ctx,
        params.collateral_amount,
        params.collateral_denom.clone(),
    )?;
    
    // Update the actual accounts with the results
    ctx.accounts.user_collateral_amount.amount = result.new_collateral_amount;
    ctx.accounts.liquidity_threshold.ratio = result.new_icr;
    ctx.accounts.state.total_debt_amount = trove_ctx.state.total_debt_amount;
    ctx.accounts.sorted_troves_state = sorted_ctx.sorted_troves_state;
    
    // Reinsert trove in sorted list based on new ICR (collateral decreases = lower ICR = riskier)
    // Note: Caller must pass remaining_accounts for reinsert operation
    if !ctx.remaining_accounts.is_empty() {
        use crate::sorted_troves_simple::reinsert_trove;
        
        reinsert_trove(
            &mut ctx.accounts.sorted_troves_state,
            &mut ctx.accounts.node,
            ctx.accounts.user.key(),
            result.new_icr,
            ctx.remaining_accounts,
        )?;
        
        msg!("Trove repositioned after collateral removal");
    } else {
        msg!("Warning: No remaining_accounts provided, skipping trove reinsert");
    }
    
    // Transfer collateral from protocol to user
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.protocol_collateral_account.to_account_info(),
            to: ctx.accounts.user_collateral_account.to_account_info(),
            authority: ctx.accounts.protocol_collateral_account.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.collateral_amount)?;

    msg!("Collateral removed successfully");
    msg!("Removed: {} {}", params.collateral_amount, params.collateral_denom);
    msg!("New collateral amount: {}", result.new_collateral_amount);
    msg!("New ICR: {}", result.new_icr);
    msg!("Debt amount: {}", result.new_debt_amount);

    Ok(())
}