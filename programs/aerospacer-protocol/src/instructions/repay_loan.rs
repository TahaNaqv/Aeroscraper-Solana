use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::msg::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Repay_loanParams {
    pub amount: u64,
}

#[derive(Accounts)]
#[instruction(params: Repay_loanParams)]
pub struct Repay_loan<'info> {
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
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_stablecoin_account: Account<'info, TokenAccount>,

    /// CHECK: This is the stable coin mint account
    pub stable_coin_mint: UncheckedAccount<'info>,

    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral", trove.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_by_denom: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Repay_loan>, params: Repay_loanParams) -> Result<()> {
    let trove = &mut ctx.accounts.trove;
    let state = &mut ctx.accounts.state;

    // Validate repayment amount
    if params.amount > trove.debt_amount {
        return Err(ErrorCode::InvalidAmount.into());
    }

    // Get collateral price from oracle using Utils
    let collateral_price = query_collateral_price(
        state.oracle_program,
        &trove.collateral_denom,
    )?;

    // Update state total debt using safe math
    state.total_debt_amount = safe_sub(state.total_debt_amount, params.amount)?;

    // Check if this is a full repayment
    if params.amount == trove.debt_amount {
        // Full repayment - close the trove
        trove.debt_amount = 0;
        trove.collateral_amount = 0;
        trove.collateral_ratio = 0;
        trove.is_active = false;

            // Update per-denom collateral total PDA
            if !ctx.accounts.total_collateral_by_denom.data_is_empty() {
                let mut data = ctx.accounts.total_collateral_by_denom.try_borrow_mut_data()?;
                let mut total_collateral: TotalCollateralByDenom = 
                    TotalCollateralByDenom::try_deserialize(&mut data.as_ref())?;
                total_collateral.total_amount = safe_sub(
                    total_collateral.total_amount,
                    trove.collateral_amount,
                )?;
                total_collateral.last_updated = Clock::get()?.unix_timestamp;
                
                total_collateral.try_serialize(&mut *data)?;
            }

        msg!("Trove fully repaid and closed");
    } else {
        // Partial repayment
        trove.debt_amount = safe_sub(trove.debt_amount, params.amount)?;
        
        // Update trove collateral ratio using Utils
        trove.collateral_ratio = calculate_collateral_ratio(
            trove.collateral_amount,
            trove.debt_amount,
            collateral_price,
        )?;

        // Validate the updated trove meets minimum collateral ratio using Utils
        validate_trove_parameters(
            &trove,
            state.minimum_collateral_ratio as u64,
            collateral_price,
        )?;

        msg!("Loan partially repaid");
        msg!("Remaining debt: {} aUSD", trove.debt_amount);
        msg!("New collateral ratio: {}%", trove.collateral_ratio / 100);
    }

    // Burn stablecoin from user
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Burn {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            from: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::burn(burn_ctx, params.amount)?;

    msg!("Loan repayment successful");
    msg!("Repaid: {} aUSD", params.amount);

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Trove not active")]
    TroveNotActive,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Overflow occurred")]
    Overflow,
}
