use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use crate::state::*;
use crate::utils::*;
use crate::error::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Withdraw_liquidation_gainsParams {
    pub collateral_denom: String,
}

#[derive(Accounts)]
#[instruction(params: Withdraw_liquidation_gainsParams)]
pub struct Withdraw_liquidation_gains<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserLiquidationCollateralGain::LEN,
        seeds = [b"user_liquidation_collateral_gain", user.key().as_ref()],
        bump
    )]
    pub user_liquidation_collateral_gain: Account<'info, UserLiquidationCollateralGain>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    /// CHECK: Protocol collateral vault PDA
    #[account(
        mut,
        seeds = [b"protocol_collateral_vault", params.collateral_denom.as_bytes()],
        bump
    )]
    pub protocol_collateral_vault: AccountInfo<'info>,

    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral_amount", params.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_amount: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    
    // Note: remaining_accounts should contain:
    // - TotalLiquidationCollateralGain PDAs for each block height with unclaimed gains
    // - UserLiquidationCollateralGain PDAs for tracking claimed status  
    // - UserStakeAmount PDA for this user
}



pub fn handler(ctx: Context<Withdraw_liquidation_gains>, params: Withdraw_liquidation_gainsParams) -> Result<()> {
    let user_liquidation_collateral_gain = &mut ctx.accounts.user_liquidation_collateral_gain;
    let state = &ctx.accounts.state;
    
    // Get user's stake amount from UserStakeAmount PDA (must be in remaining_accounts[0])
    require!(
        !ctx.remaining_accounts.is_empty(),
        AerospacerProtocolError::InvalidList
    );
    
    // SECURITY: Verify the UserStakeAmount PDA is legitimate
    let user_key = ctx.accounts.user.key();
    let user_stake_seeds = UserStakeAmount::seeds(&user_key);
    let (expected_stake_pda, _bump) = Pubkey::find_program_address(&user_stake_seeds, &crate::ID);
    
    let stake_account_info = &ctx.remaining_accounts[0];
    require!(
        stake_account_info.key() == expected_stake_pda,
        AerospacerProtocolError::Unauthorized
    );
    
    // Read stake amount from the verified account data
    let user_stake_amount = {
        let data = stake_account_info.try_borrow_data()?;
        // Skip discriminator (8 bytes) and read the amount field (u64)
        let amount_bytes = &data[8..16];
        u64::from_le_bytes(amount_bytes.try_into().unwrap())
    };
    let total_stake = state.total_stake_amount;
    
    // Check if user has any stake
    require!(
        user_stake_amount > 0,
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        total_stake > 0,
        AerospacerProtocolError::InvalidAmount
    );
    
    // Get SPL token balance from vault (not lamports - that's just rent!)
    // The vault is a TokenAccount, so deserialize it to read the amount field
    let vault_token_balance = {
        let vault_data = ctx.accounts.protocol_collateral_vault.try_borrow_data()?;
        // TokenAccount.amount is at offset 64 (after mint: 32, owner: 32)
        let amount_bytes = &vault_data[64..72];
        u64::from_le_bytes(amount_bytes.try_into().unwrap())
    };
    
    // Calculate user's proportional share of vault token balance
    // proportional_share = (user_stake / total_stake) * vault_token_balance
    let proportional_share = (user_stake_amount as u128)
        .checked_mul(vault_token_balance as u128)
        .ok_or(AerospacerProtocolError::OverflowError)?
        .checked_div(total_stake as u128)
        .ok_or(AerospacerProtocolError::DivideByZeroError)?
        as u64;
    
    // Check if user has any gains to withdraw
    require!(
        proportional_share > 0,
        AerospacerProtocolError::CollateralRewardsNotFound
    );
    
    msg!("Withdrawing liquidation gains:");
    msg!("  User stake: {} / {} total", user_stake_amount, total_stake);
    msg!("  Vault token balance: {}", vault_token_balance);
    msg!("  Proportional share: {}", proportional_share);
    
    // Update liquidation gains account to mark as claimed
    user_liquidation_collateral_gain.user = ctx.accounts.user.key();
    user_liquidation_collateral_gain.block_height = Clock::get()?.slot;
    user_liquidation_collateral_gain.claimed = true;
    
    let total_gain = proportional_share;

    // Transfer collateral from protocol to user
    let transfer_seeds = &[
        b"protocol_collateral_vault".as_ref(),
        params.collateral_denom.as_bytes(),
        &[ctx.bumps.protocol_collateral_vault],
    ];
    let transfer_signer = &[&transfer_seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.protocol_collateral_vault.to_account_info(),
            to: ctx.accounts.user_collateral_account.to_account_info(),
            authority: ctx.accounts.protocol_collateral_vault.to_account_info(),
        },
        transfer_signer,
    );
    anchor_spl::token::transfer(transfer_ctx, total_gain)?;

    // Update per-denom collateral total PDA
    update_total_collateral_from_account_info(
        &ctx.accounts.total_collateral_amount,
        -(total_gain as i64),
    )?;

    msg!("Liquidation gains withdrawn successfully");
    msg!("Amount: {} {}", total_gain, params.collateral_denom);
    msg!("User: {}", ctx.accounts.user.key());

    Ok(())
}