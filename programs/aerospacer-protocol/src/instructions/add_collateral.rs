use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use aerospacer_utils::{self, *};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Add_collateralParams {
    pub amount: u64,
    pub collateral_denom: String,
}

#[derive(Accounts)]
#[instruction(params: Add_collateralParams)]
pub struct Add_collateral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"trove", user.key().as_ref()],
        bump,
        constraint = trove.owner == user.key() @ ErrorCode::Unauthorized,
        constraint = trove.is_active @ ErrorCode::TroveNotActive
    )]
    pub trove: Account<'info, TroveAccount>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_collateral_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Add_collateral>, params: Add_collateralParams) -> Result<()> {
    let trove = &mut ctx.accounts.trove;
    let state = &mut ctx.accounts.state;

    // Validate collateral denom matches trove
    if trove.collateral_denom != params.collateral_denom {
        return Err(ErrorCode::InvalidCollateralDenom.into());
    }

    // Validate collateral parameters using Utils
    validate_collateral_params(
        params.amount,
        ctx.accounts.user_collateral_account.mint,
        aerospacer_utils::MINIMUM_LOAN_AMOUNT / 1000, // Minimum amount for adding collateral
    )?;

    // Calculate protocol fee using Utils
    let protocol_fee_amount = calculate_protocol_fee(
        params.amount,
        state.protocol_fee as u64,
    )?;

    let net_collateral_amount = calculate_net_amount(
        params.amount,
        state.protocol_fee as u64,
    )?;

    // Get collateral price from oracle using Utils
    let collateral_price = query_collateral_price(
        state.oracle_program,
        params.collateral_denom.clone(),
    )?;

    // Update trove collateral using safe math
    let new_collateral_amount = safe_add(trove.collateral_amount, net_collateral_amount)?;

    trove.collateral_amount = new_collateral_amount;
    trove.collateral_ratio = calculate_collateral_ratio(
        trove.collateral_amount,
        trove.debt_amount,
        collateral_price,
    )?;

    // Validate the updated trove meets minimum collateral ratio using Utils
    validate_trove_parameters(
        trove.collateral_amount,
        trove.debt_amount,
        state.minimum_collateral_ratio as u64,
        collateral_price,
    )?;

    // Update state totals using safe math
    if let Some(index) = state.collateral_denoms.iter().position(|d| d == &params.collateral_denom) {
        state.total_collateral_amounts[index] = safe_add(
            state.total_collateral_amounts[index],
            net_collateral_amount,
        )?;
    } else {
        return Err(ErrorCode::InvalidCollateralDenom.into());
    }

    // Transfer collateral from user to protocol
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: ctx.accounts.user_collateral_account.to_account_info(),
            to: ctx.accounts.protocol_collateral_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.amount)?;

    // Process protocol fees using Utils
    process_protocol_fees(
        state.fee_distributor,
        protocol_fee_amount,
        params.collateral_denom.clone(),
    )?;

    msg!("Collateral added successfully");
    msg!("Added: {} {}", net_collateral_amount, params.collateral_denom);
    msg!("Protocol fee: {} {}", protocol_fee_amount, params.collateral_denom);
    msg!("New collateral ratio: {}%", trove.collateral_ratio / 100);

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Trove not active")]
    TroveNotActive,
    #[msg("Invalid collateral denom")]
    InvalidCollateralDenom,
    #[msg("Overflow occurred")]
    Overflow,
}
