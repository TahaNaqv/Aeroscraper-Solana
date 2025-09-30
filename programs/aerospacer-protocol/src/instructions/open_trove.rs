use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::utils::*;
use crate::error::AerospacerProtocolError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Open_troveParams {
    pub loan_amount: u64,
    pub collateral_amount: u64,
    pub collateral_denom: String,
}

#[derive(Accounts)]
#[instruction(params: Open_troveParams)]
pub struct Open_trove<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + TroveAccount::LEN,
        seeds = [b"trove", user.key().as_ref()],
        bump
    )]
    pub trove: Account<'info, TroveAccount>,

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
        seeds = [b"total_collateral", params.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_by_denom: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Open_trove>, params: Open_troveParams) -> Result<()> {
    let trove = &mut ctx.accounts.trove;
    let state = &mut ctx.accounts.state;

    // Use local validation functions
    if params.loan_amount < crate::state::MINIMUM_LOAN_AMOUNT {
        return Err(AerospacerProtocolError::LoanAmountBelowMinimum.into());
    }

    // Validate collateral parameters
    if params.collateral_amount < crate::state::MINIMUM_COLLATERAL_AMOUNT {
        return Err(AerospacerProtocolError::CollateralBelowMinimum.into());
    }

    // Get collateral price from oracle
    let collateral_price = query_collateral_price(
        state.oracle_program,
        &params.collateral_denom,
    )?;

    // Calculate protocol fee
    let protocol_fee_amount = calculate_protocol_fee(
        params.collateral_amount,
        state.protocol_fee,
    )?;

    let net_collateral_amount = calculate_net_amount(
        params.collateral_amount,
        state.protocol_fee,
    )?;

    // Validate trove parameters
    validate_trove_parameters(
        &trove,
        state.minimum_collateral_ratio as u64,
        collateral_price,
    )?;

    // Update trove
    trove.owner = ctx.accounts.user.key();
    trove.debt_amount = params.loan_amount;
    trove.collateral_amount = net_collateral_amount;
    trove.collateral_ratio = calculate_collateral_ratio(
        net_collateral_amount,
        params.loan_amount,
        collateral_price,
    )?;
    trove.collateral_denom = params.collateral_denom.clone();
    trove.is_active = true;
    trove.created_at = Clock::get()?.unix_timestamp;

    // Update state totals using safe math
    state.total_debt_amount = safe_add(state.total_debt_amount, params.loan_amount)?;

    // Add denom to supported list if not already present
    if !state.collateral_denoms.contains(&params.collateral_denom) {
        state.collateral_denoms.push(params.collateral_denom.clone());
    }

    // Initialize or update per-denom collateral total PDA
    if ctx.accounts.total_collateral_by_denom.data_is_empty() {
        // Initialize the PDA account
        let total_collateral = TotalCollateralByDenom {
            denom: params.collateral_denom.clone(),
            total_amount: net_collateral_amount,
            last_updated: Clock::get()?.unix_timestamp,
        };
        
        let mut data = ctx.accounts.total_collateral_by_denom.try_borrow_mut_data()?;
        total_collateral.try_serialize(&mut *data)?;
    } else {
        // Update existing PDA account
        let mut data = ctx.accounts.total_collateral_by_denom.try_borrow_mut_data()?;
        let mut total_collateral: TotalCollateralByDenom = 
            TotalCollateralByDenom::try_deserialize(&mut data.as_ref())?;
        total_collateral.total_amount = safe_add(
            total_collateral.total_amount,
            net_collateral_amount,
        )?;
        total_collateral.last_updated = Clock::get()?.unix_timestamp;
        
        total_collateral.try_serialize(&mut *data)?;
    }

    // Transfer collateral from user to protocol
    let transfer_collateral_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: ctx.accounts.user_collateral_account.to_account_info(),
            to: ctx.accounts.protocol_collateral_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_collateral_ctx, params.collateral_amount)?;

    // Mint stablecoin to user
    let mint_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::MintTo {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.protocol_stablecoin_account.to_account_info(),
        },
    );
    anchor_spl::token::mint_to(mint_ctx, params.loan_amount)?;

    // Process protocol fees
    process_protocol_fees(
        state.fee_distributor,
        protocol_fee_amount,
        params.collateral_denom.clone(),
    )?;

    msg!("Trove opened successfully");
    msg!("Collateral: {} {}", net_collateral_amount, params.collateral_denom);
    msg!("Debt: {} aUSD", params.loan_amount);
    msg!("Protocol fee: {} {}", protocol_fee_amount, params.collateral_denom);

    Ok(())
}

// Constants are defined in crate::state

