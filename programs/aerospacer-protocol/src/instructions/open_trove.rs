use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use aerospacer_utils::{self, *};

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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Open_trove>, params: Open_troveParams) -> Result<()> {
    let trove = &mut ctx.accounts.trove;
    let state = &mut ctx.accounts.state;

    // Use Utils Library for validation
    if params.loan_amount < aerospacer_utils::MINIMUM_LOAN_AMOUNT {
        return Err(ErrorCode::LoanAmountBelowMinimum.into());
    }

    // Validate collateral parameters using Utils
    validate_collateral_params(
        params.collateral_amount,
        ctx.accounts.user_collateral_account.mint,
        aerospacer_utils::MINIMUM_LOAN_AMOUNT / 100, // Minimum collateral amount
    )?;

    // Get collateral price from oracle using Utils
    let collateral_price = query_collateral_price(
        state.oracle_program,
        params.collateral_denom.clone(),
    )?;

    // Calculate protocol fee using Utils
    let protocol_fee_amount = calculate_protocol_fee(
        params.collateral_amount,
        state.protocol_fee as u64,
    )?;

    let net_collateral_amount = calculate_net_amount(
        params.collateral_amount,
        state.protocol_fee as u64,
    )?;

    // Validate trove parameters using Utils
    validate_trove_parameters(
        net_collateral_amount,
        params.loan_amount,
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

    // Update collateral totals (simplified - in real implementation you'd use a map)
    if let Some(index) = state.collateral_denoms.iter().position(|d| d == &params.collateral_denom) {
        state.total_collateral_amounts[index] = safe_add(
            state.total_collateral_amounts[index],
            net_collateral_amount,
        )?;
    } else {
        state.collateral_denoms.push(params.collateral_denom.clone());
        state.total_collateral_amounts.push(net_collateral_amount);
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

    // Process protocol fees using Utils
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

#[error_code]
pub enum ErrorCode {
    #[msg("Loan amount below minimum")]
    LoanAmountBelowMinimum,
    #[msg("Trove already exists")]
    TroveExists,
    #[msg("Overflow occurred")]
    Overflow,
}
