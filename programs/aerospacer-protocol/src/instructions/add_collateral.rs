use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::utils::*;
use crate::error::*;

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

    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral", params.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_by_denom: AccountInfo<'info>,

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
        MINIMUM_COLLATERAL_AMOUNT, // Minimum amount for adding collateral
    )?;

    // Calculate protocol fee using Utils
    let protocol_fee_amount = calculate_protocol_fee(
        params.amount,
        state.protocol_fee,
    )?;

    let net_collateral_amount = calculate_net_amount(
        params.amount,
        state.protocol_fee,
    )?;

    // Get collateral price from oracle using Utils
    let collateral_price = query_collateral_price(
        state.oracle_program,
        &params.collateral_denom,
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
        trove,
        state.minimum_collateral_ratio as u64,
        collateral_price,
    )?;

        // Update per-denom collateral total PDA
        if !ctx.accounts.total_collateral_by_denom.data_is_empty() {
            let mut data = ctx.accounts.total_collateral_by_denom.try_borrow_mut_data()?;
            let mut total_collateral: TotalCollateralByDenom = 
                TotalCollateralByDenom::try_deserialize(&mut data.as_ref())?;
            total_collateral.total_amount = safe_add(
                total_collateral.total_amount,
                net_collateral_amount,
            )?;
            total_collateral.last_updated = Clock::get()?.unix_timestamp;
            
            total_collateral.try_serialize(&mut *data)?;
        } else {
            return Err(AerospacerProtocolError::InvalidCollateralDenom.into());
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
