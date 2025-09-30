use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::msg::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Borrow_loanParams {
    pub amount: u64,
}

#[derive(Accounts)]
#[instruction(params: Borrow_loanParams)]
pub struct Borrow_loan<'info> {
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

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Borrow_loan>, params: Borrow_loanParams) -> Result<()> {
    let trove = &mut ctx.accounts.trove;
    let state = &mut ctx.accounts.state;

    // Validate loan amount using Utils
    if params.amount < crate::state::MINIMUM_LOAN_AMOUNT {
        return Err(ErrorCode::LoanAmountBelowMinimum.into());
    }

    // Get collateral price from oracle using Utils
    let collateral_price = query_collateral_price(
        state.oracle_program,
        &trove.collateral_denom,
    )?;

    // Update trove debt using safe math
    let new_debt_amount = safe_add(trove.debt_amount, params.amount)?;
    trove.debt_amount = new_debt_amount;

    // Update state total debt using safe math
    state.total_debt_amount = safe_add(state.total_debt_amount, params.amount)?;

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

    // Mint stablecoin to user
    let mint_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::MintTo {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.protocol_stablecoin_account.to_account_info(),
        },
    );
    anchor_spl::token::mint_to(mint_ctx, params.amount)?;

    msg!("Loan borrowed successfully");
    msg!("Borrowed: {} aUSD", params.amount);
    msg!("New debt amount: {} aUSD", new_debt_amount);
    msg!("New collateral ratio: {}%", trove.collateral_ratio / 100);

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Trove not active")]
    TroveNotActive,
    #[msg("Loan amount below minimum")]
    LoanAmountBelowMinimum,
    #[msg("Overflow occurred")]
    Overflow,
}
