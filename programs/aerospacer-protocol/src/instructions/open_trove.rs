use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, MintTo};
use crate::state::*;
use crate::error::*;
use crate::account_management::*;
use crate::oracle::*;
use crate::trove_management::TroveManager;
use crate::state::{MINIMUM_LOAN_AMOUNT, MINIMUM_COLLATERAL_AMOUNT};
use crate::fees_integration::*;
use crate::utils::*;
use crate::sorted_troves;

// Oracle integration is now handled via our aerospacer-oracle contract

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct OpenTroveParams {
    pub loan_amount: u64,
    pub collateral_denom: String,
    pub collateral_amount: u64,
}

#[derive(Accounts)]
#[instruction(params: OpenTroveParams)]
pub struct OpenTrove<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    // Trove context accounts - Box<> to reduce stack usage
    #[account(
        init,
        payer = user,
        space = 8 + UserDebtAmount::LEN,
        seeds = [b"user_debt_amount", user.key().as_ref()],
        bump
    )]
    pub user_debt_amount: Box<Account<'info, UserDebtAmount>>,
    
    #[account(
        init,
        payer = user,
        space = 8 + LiquidityThreshold::LEN,
        seeds = [b"liquidity_threshold", user.key().as_ref()],
        bump
    )]
    pub liquidity_threshold: Box<Account<'info, LiquidityThreshold>>,
    
    // Collateral context accounts
    #[account(
        init,
        payer = user,
        space = 8 + UserCollateralAmount::LEN,
        seeds = [b"user_collateral_amount", user.key().as_ref(), params.collateral_denom.as_bytes()],
        bump
    )]
    pub user_collateral_amount: Box<Account<'info, UserCollateralAmount>>,
    
    #[account(
        mut,
        constraint = user_collateral_account.owner == user.key() @ AerospacerProtocolError::Unauthorized,
        constraint = user_collateral_account.mint == collateral_mint.key() @ AerospacerProtocolError::InvalidMint
    )]
    pub user_collateral_account: Box<Account<'info, TokenAccount>>,
    
    pub collateral_mint: Account<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = user,
        token::mint = collateral_mint,
        token::authority = protocol_collateral_account,
        seeds = [b"protocol_collateral_vault", params.collateral_denom.as_bytes()],
        bump
    )]
    pub protocol_collateral_account: Box<Account<'info, TokenAccount>>,
    
    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral_amount", params.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_amount: AccountInfo<'info>,
    
    // Sorted troves context accounts - Box<> to reduce stack usage
    #[account(
        mut,
        seeds = [b"sorted_troves_state"],
        bump
    )]
    pub sorted_troves_state: Box<Account<'info, SortedTrovesState>>,
    
    // Node account for sorted troves linked list
    #[account(
        init,
        payer = user,
        space = 8 + Node::LEN,
        seeds = [b"node", user.key().as_ref()],
        bump
    )]
    pub node: Box<Account<'info, Node>>,
    
    // State account - Box<> to reduce stack usage
    #[account(mut)]
    pub state: Box<Account<'info, StateAccount>>,
    
    // Token accounts - Box<> to reduce stack usage
    #[account(
        mut,
        constraint = user_stablecoin_account.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_stablecoin_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        init_if_needed,
        payer = user,
        token::mint = stable_coin_mint,
        token::authority = protocol_stablecoin_account,
        seeds = [b"protocol_stablecoin_vault"],
        bump
    )]
    pub protocol_stablecoin_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        constraint = stable_coin_mint.key() == state.stable_coin_addr @ AerospacerProtocolError::InvalidMint
    )]
    pub stable_coin_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    
    // Oracle and fee accounts moved to remaining_accounts to reduce stack usage:
    // remaining_accounts[0] = oracle_program
    // remaining_accounts[1] = oracle_state
    // remaining_accounts[2] = pyth_price_account
    // remaining_accounts[3] = clock
    // remaining_accounts[4] = fees_program
    // remaining_accounts[5] = fees_state
    // remaining_accounts[6] = stability_pool_token_account
    // remaining_accounts[7] = fee_address_1_token_account
    // remaining_accounts[8] = fee_address_2_token_account
}

pub fn handler(ctx: Context<OpenTrove>, params: OpenTroveParams) -> Result<()> {
    // Extract oracle and fee accounts from remaining_accounts to reduce stack usage
    require!(
        ctx.remaining_accounts.len() >= 9,
        AerospacerProtocolError::InvalidAccountData
    );
    
    let oracle_program = &ctx.remaining_accounts[0];
    let oracle_state = &ctx.remaining_accounts[1];
    let pyth_price_account = &ctx.remaining_accounts[2];
    let clock = Clock::from_account_info(&ctx.remaining_accounts[3])?;
    let fees_program = &ctx.remaining_accounts[4];
    let fees_state = &ctx.remaining_accounts[5];
    let stability_pool_token_account = &ctx.remaining_accounts[6];
    let fee_address_1_token_account = &ctx.remaining_accounts[7];
    let fee_address_2_token_account = &ctx.remaining_accounts[8];
    
    // Validate oracle accounts
    require!(
        oracle_program.key() == ctx.accounts.state.oracle_helper_addr,
        AerospacerProtocolError::Unauthorized
    );
    require!(
        oracle_state.key() == ctx.accounts.state.oracle_state_addr,
        AerospacerProtocolError::Unauthorized
    );
    
    // Validate fee accounts
    require!(
        fees_program.key() == ctx.accounts.state.fee_distributor_addr,
        AerospacerProtocolError::Unauthorized
    );
    require!(
        fees_state.key() == ctx.accounts.state.fee_state_addr,
        AerospacerProtocolError::Unauthorized
    );
    
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
    
    // Calculate opening fee BEFORE trove operations
    let fee_amount = calculate_protocol_fee(params.loan_amount, ctx.accounts.state.protocol_fee)?;
    let net_loan_amount = params.loan_amount.saturating_sub(fee_amount);
    
    msg!("Opening fee: {} aUSD ({}%)", fee_amount, ctx.accounts.state.protocol_fee);
    msg!("Net loan amount: {} aUSD", net_loan_amount);
    
    // Create contexts in scoped block to reduce stack usage
    // Execute trove operations and capture results
    let result = {
        let mut trove_ctx = TroveContext {
            user: ctx.accounts.user.clone(),
            user_debt_amount: (*ctx.accounts.user_debt_amount).clone(),
            liquidity_threshold: (*ctx.accounts.liquidity_threshold).clone(),
            state: (*ctx.accounts.state).clone(),
        };
        
        let mut collateral_ctx = CollateralContext {
            user: ctx.accounts.user.clone(),
            user_collateral_amount: (*ctx.accounts.user_collateral_amount).clone(),
            user_collateral_account: (*ctx.accounts.user_collateral_account).clone(),
            protocol_collateral_account: (*ctx.accounts.protocol_collateral_account).clone(),
            total_collateral_amount: ctx.accounts.total_collateral_amount.clone(),
            token_program: ctx.accounts.token_program.clone(),
        };
        
        let oracle_ctx = OracleContext {
            oracle_program: oracle_program.clone(),
            oracle_state: oracle_state.clone(),
            pyth_price_account: pyth_price_account.clone(),
            clock: ctx.remaining_accounts[3].clone(),
        };
        
        // Use TroveManager with NET loan amount (after fee)
        let result = TroveManager::open_trove(
            &mut trove_ctx,
            &mut collateral_ctx,
            &oracle_ctx,
            net_loan_amount,  // Use net amount for debt recording
            params.collateral_amount,
            params.collateral_denom.clone(),
        )?;
        
        // Update state total debt before contexts are dropped
        ctx.accounts.state.total_debt_amount = trove_ctx.state.total_debt_amount;
        
        Ok::<_, Error>(result)
    }?;
    
    // Update the actual accounts with the results
    ctx.accounts.user_debt_amount.amount = result.new_debt_amount;
    ctx.accounts.liquidity_threshold.ratio = result.new_icr;
    ctx.accounts.user_collateral_amount.amount = result.new_collateral_amount;
    
    // Insert trove into sorted list using the Node account from context
    // Pass remaining_accounts[9..] for sorted troves operations (first 9 are oracle/fee accounts)
    sorted_troves::insert_trove(
        &mut *ctx.accounts.sorted_troves_state,
        &mut *ctx.accounts.node,
        ctx.accounts.user.key(),
        result.new_icr,
        &ctx.remaining_accounts[9..],
    )?;
    
    // Mint full loan amount to user first (user requested full amount, will pay fee from it)
    // Use invoke_signed for PDA authority
    let mint_seeds = &[
        b"protocol_stablecoin_vault".as_ref(),
        &[ctx.bumps.protocol_stablecoin_account],
    ];
    let mint_signer = &[&mint_seeds[..]];
    
    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.protocol_stablecoin_account.to_account_info(),
        },
        mint_signer,
    );
    anchor_spl::token::mint_to(mint_ctx, params.loan_amount)?;
    
    // Distribute opening fee via CPI to aerospacer-fees
    if fee_amount > 0 {
        let _net_amount = process_protocol_fee(
            params.loan_amount,
            ctx.accounts.state.protocol_fee,
            fees_program.to_account_info(),
            ctx.accounts.user.to_account_info(),
            fees_state.to_account_info(),
            ctx.accounts.user_stablecoin_account.to_account_info(),
            stability_pool_token_account.to_account_info(),
            fee_address_1_token_account.to_account_info(),
            fee_address_2_token_account.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        )?;
        
        msg!("Opening fee collected and distributed: {} aUSD", fee_amount);
        msg!("Net loan amount after fee: {} aUSD", net_loan_amount);
    }
    
    // Log success
    msg!("Trove opened successfully");
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Loan amount: {} aUSD (fee: {})", params.loan_amount, fee_amount);
    msg!("Collateral: {} {}", params.collateral_amount, params.collateral_denom);
    msg!("ICR: {}", result.new_icr);
    
    Ok(())
}