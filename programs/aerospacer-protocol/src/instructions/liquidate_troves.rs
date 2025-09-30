use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, Burn};
use crate::state::*;
use crate::utils::*;
use crate::error::AerospacerProtocolError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LiquidateTrovesParams {
    pub trove_owner: Pubkey, // Owner of the single trove to liquidate
}

#[derive(Accounts)]
#[instruction(params: LiquidateTrovesParams)]
pub struct LiquidateTroves<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"trove", params.trove_owner.as_ref()],
        bump,
        constraint = trove.is_active @ AerospacerProtocolError::TroveNotActive
    )]
    pub trove: Account<'info, TroveAccount>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub stable_coin_mint: Account<'info, Mint>,

    #[account(mut)]
    pub liquidator_stablecoin_account: Account<'info, TokenAccount>,

    /// CHECK: Protocol stablecoin vault PDA
    #[account(
        mut,
        seeds = [b"protocol_stablecoin_vault"],
        bump
    )]
    pub protocol_stablecoin_vault: AccountInfo<'info>,
    
    /// CHECK: Protocol collateral vault PDA
    #[account(
        mut,
        seeds = [b"protocol_collateral_vault", trove.collateral_denom.as_bytes()],
        bump
    )]
    pub protocol_collateral_vault: AccountInfo<'info>,

    #[account(mut)]
    pub liquidator_collateral_account: Account<'info, TokenAccount>,
    
    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral", trove.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_by_denom: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<LiquidateTroves>, params: LiquidateTrovesParams) -> Result<()> {
    let trove = &mut ctx.accounts.trove;
    let state = &mut ctx.accounts.state;
    let liquidator = &ctx.accounts.liquidator;

    // Get collateral price from oracle
    let collateral_price = query_collateral_price(state.oracle_program, &trove.collateral_denom)?;

    // Check if trove is liquidatable
    let liquidatable = crate::trove_helpers::can_liquidate_trove(
        trove.collateral_amount,
        trove.debt_amount,
        collateral_price,
        state.minimum_collateral_ratio as u64,
    )?;

    require!(liquidatable, AerospacerProtocolError::TroveNotLiquidatable);

    // Calculate collateral reward (excess collateral over debt value)
    let collateral_reward = crate::trove_helpers::calculate_liquidation_reward(
        trove.debt_amount,
        trove.collateral_amount,
        collateral_price,
    )?;

    // Transfer stablecoins from liquidator to protocol (to cover debt)
    if trove.debt_amount > 0 {
        let transfer_sc_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.liquidator_stablecoin_account.to_account_info(),
                to: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
                authority: ctx.accounts.liquidator.to_account_info(),
            },
        );
        anchor_spl::token::transfer(transfer_sc_ctx, trove.debt_amount)?;

        // Burn from protocol stablecoin account
        let burn_seeds = &[
            b"protocol_stablecoin_vault".as_ref(),
            &[ctx.bumps.protocol_stablecoin_vault],
        ];
        let burn_signer = &[&burn_seeds[..]];

        let burn_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.stable_coin_mint.to_account_info(),
                from: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
                authority: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
            },
            burn_signer,
        );
        anchor_spl::token::burn(burn_ctx, trove.debt_amount)?;
    }

    // Transfer collateral reward to liquidator
    if collateral_reward > 0 {
        let transfer_col_seeds = &[
            b"protocol_collateral_vault".as_ref(),
            trove.collateral_denom.as_bytes(),
            &[ctx.bumps.protocol_collateral_vault],
        ];
        let transfer_col_signer = &[&transfer_col_seeds[..]];

        let transfer_col_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.protocol_collateral_vault.to_account_info(),
                to: ctx.accounts.liquidator_collateral_account.to_account_info(),
                authority: ctx.accounts.protocol_collateral_vault.to_account_info(),
            },
            transfer_col_signer,
        );
        anchor_spl::token::transfer(transfer_col_ctx, collateral_reward)?;
    }

    // Update state totals
    state.total_debt_amount = safe_sub(state.total_debt_amount, trove.debt_amount)?;
    
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

    // Mark trove closed
    trove.debt_amount = 0;
    trove.collateral_amount = 0;
    trove.collateral_ratio = 0;
    trove.is_active = false;

    msg!("Trove liquidated successfully for owner: {}", params.trove_owner);
    msg!("Debt covered: {} aUSD", trove.debt_amount);
    msg!("Collateral reward to liquidator: {} {}", collateral_reward, trove.collateral_denom);

    Ok(())
}

