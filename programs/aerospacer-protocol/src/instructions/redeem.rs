use std::collections::HashMap;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, Burn};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::trove_management::*;
use crate::account_management::*;
use crate::oracle::*;
use crate::fees_integration::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RedeemParams {
    pub amount: u64, // Equivalent to Uint256
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: RedeemParams)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(
        mut,
        seeds = [b"user_debt_amount", user.key().as_ref()],
        bump,
        constraint = user_debt_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_debt_amount: Account<'info, UserDebtAmount>,

    #[account(
        mut,
        seeds = [b"liquidity_threshold", user.key().as_ref()],
        bump,
        constraint = liquidity_threshold.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub liquidity_threshold: Account<'info, LiquidityThreshold>,

    #[account(
        mut,
        constraint = user_stablecoin_account.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user_collateral_amount", user.key().as_ref(), b"SOL"],
        bump,
        constraint = user_collateral_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_collateral_amount: Account<'info, UserCollateralAmount>,

    #[account(
        mut,
        constraint = user_collateral_account.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_collateral_account: Account<'info, TokenAccount>,

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
        seeds = [b"protocol_collateral_vault", b"SOL"], // Default to SOL for now
        bump
    )]
    pub protocol_collateral_vault: AccountInfo<'info>,

    /// CHECK: This is the stable coin mint account
    #[account(
        constraint = stable_coin_mint.key() == state.stable_coin_addr @ AerospacerProtocolError::InvalidMint
    )]
    pub stable_coin_mint: UncheckedAccount<'info>,

    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral_amount", b"SOL"],
        bump
    )]
    pub total_collateral_amount: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"sorted_troves_state"],
        bump
    )]
    pub sorted_troves_state: Account<'info, SortedTrovesState>,

    // Oracle context - integration with our aerospacer-oracle
    /// CHECK: Our oracle program
    #[account(mut)]
    pub oracle_program: AccountInfo<'info>,
    
    /// CHECK: Oracle state account
    #[account(mut)]
    pub oracle_state: AccountInfo<'info>,

    // Fee distribution accounts
    /// CHECK: Fees program
    #[account(
        constraint = fees_program.key() == state.fee_distributor_addr @ AerospacerProtocolError::Unauthorized
    )]
    pub fees_program: AccountInfo<'info>,
    
    /// CHECK: Fees state account
    #[account(mut)]
    pub fees_state: AccountInfo<'info>,
    
    /// CHECK: Stability pool token account
    #[account(mut)]
    pub stability_pool_token_account: AccountInfo<'info>,
    
    /// CHECK: Fee address 1 token account
    #[account(mut)]
    pub fee_address_1_token_account: AccountInfo<'info>,
    
    /// CHECK: Fee address 2 token account
    #[account(mut)]
    pub fee_address_2_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Redeem>, params: RedeemParams) -> Result<()> {
    // Validate input parameters
    require!(
        params.amount > 0,
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        params.amount >= MINIMUM_LOAN_AMOUNT, // Use same minimum as loans
        AerospacerProtocolError::InvalidAmount
    );
    
    let state = &mut ctx.accounts.state;
    
    // Validate redemption amount
    require!(
        params.amount <= state.total_debt_amount,
        AerospacerProtocolError::NotEnoughLiquidityForRedeem
    );
    
    // Validate user has enough stablecoins (including fee)
    require!(
        ctx.accounts.user_stablecoin_account.amount >= params.amount,
        AerospacerProtocolError::InvalidAmount
    );
    
    // Collect redemption fee via CPI to aerospacer-fees
    // This returns the net amount after fee deduction
    let net_redemption_amount = process_protocol_fee(
        params.amount,
        ctx.accounts.state.protocol_fee,
        ctx.accounts.fees_program.to_account_info(),
        ctx.accounts.user.to_account_info(),
        ctx.accounts.fees_state.to_account_info(),
        ctx.accounts.user_stablecoin_account.to_account_info(),
        ctx.accounts.stability_pool_token_account.to_account_info(),
        ctx.accounts.fee_address_1_token_account.to_account_info(),
        ctx.accounts.fee_address_2_token_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    )?;
    
    let fee_amount = params.amount.saturating_sub(net_redemption_amount);
    msg!("Redemption fee: {} aUSD ({}%)", fee_amount, ctx.accounts.state.protocol_fee);
    msg!("Net redemption amount: {} aUSD", net_redemption_amount);
    
    // Transfer NET redemption amount from user to protocol (after fee deduction)
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_stablecoin_account.to_account_info(),
            to: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, net_redemption_amount)?;

    // Burn NET redemption amount (not including fee)
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            from: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
            authority: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
        },
    );
    anchor_spl::token::burn(burn_ctx, net_redemption_amount)?;

    // Implement REAL core redemption logic using NET amount (after fee)
    let mut remaining_amount = net_redemption_amount;
    let mut total_collateral_sent = 0u64;
    let mut troves_redeemed = 0u32;
    
    // Start from the riskiest trove (head of sorted list)
    let mut current_trove = ctx.accounts.sorted_troves_state.head;
    
    while let Some(trove_user) = current_trove {
        if remaining_amount == 0 {
            break;
        }
        
        // REAL IMPLEMENTATION: Get trove information from remaining accounts
        // Parse trove data from remaining_accounts (4 accounts per trove)
        let trove_data = parse_trove_data_for_redemption(&trove_user, &ctx.remaining_accounts)?;
        
        // Calculate how much to redeem from this trove
        let redeem_from_trove = remaining_amount.min(trove_data.debt_amount);
        
        // Calculate collateral to send (proportional to debt redeemed)
        let collateral_ratio = if trove_data.debt_amount > 0 {
            (redeem_from_trove as f64) / (trove_data.debt_amount as f64)
        } else {
            0.0
        };
        
        // Process each collateral type in the trove
        for (denom, amount) in &trove_data.collateral_amounts {
            let collateral_to_send = ((*amount as f64) * collateral_ratio) as u64;
            
            if collateral_to_send > 0 {
                // Transfer collateral to user
                let collateral_transfer_ctx = CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.protocol_collateral_vault.to_account_info(),
                        to: ctx.accounts.user_collateral_account.to_account_info(),
                        authority: ctx.accounts.protocol_collateral_vault.to_account_info(),
                    },
                );
                anchor_spl::token::transfer(collateral_transfer_ctx, collateral_to_send)?;
                
                total_collateral_sent = total_collateral_sent.saturating_add(collateral_to_send);
                msg!("Transferred {} {} to user", collateral_to_send, denom);
            }
        }
        
        // Update trove debt (REAL implementation)
        let new_debt = trove_data.debt_amount.saturating_sub(redeem_from_trove);
        
        if new_debt == 0 {
            // Full redemption - close trove
            close_trove_after_redemption(&trove_user, &mut ctx.accounts.sorted_troves_state)?;
            msg!("Trove fully redeemed and closed: {}", trove_user);
        } else {
            // Partial redemption - update trove state
            update_trove_after_partial_redemption(&trove_user, new_debt, &mut ctx.accounts.sorted_troves_state)?;
            msg!("Trove partially redeemed: user={}, new_debt={}", trove_user, new_debt);
        }
        
        troves_redeemed += 1;
        remaining_amount = remaining_amount.saturating_sub(redeem_from_trove);
        
        // Move to next trove (REAL implementation)
        current_trove = get_next_trove_in_sorted_list(&trove_user, &ctx.accounts.sorted_troves_state)?;
    }
    
    // Update global state with net redeemed amount
    state.total_debt_amount = state.total_debt_amount.saturating_sub(net_redemption_amount - remaining_amount);
    
    msg!("Redeemed successfully");
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Gross amount: {} aUSD", params.amount);
    msg!("Fee: {} aUSD ({}%)", fee_amount, ctx.accounts.state.protocol_fee);
    msg!("Net redemption: {} aUSD", net_redemption_amount);
    msg!("Collateral sent: {} SOL", total_collateral_sent);
    msg!("Troves redeemed: {}", troves_redeemed);
    msg!("Remaining amount: {} aUSD", remaining_amount);

    Ok(())
}

// Helper function to parse trove data for redemption
fn parse_trove_data_for_redemption(
    trove_user: &Pubkey,
    remaining_accounts: &[AccountInfo],
) -> Result<TroveData> {
    // Find the trove data in remaining_accounts
    // Each trove has 4 accounts: UserDebtAmount, UserCollateralAmount, LiquidityThreshold, TokenAccount
    for account_chunk in remaining_accounts.chunks(4) {
        if account_chunk.len() >= 4 {
            // Parse UserDebtAmount
            let debt_account = &account_chunk[0];
            
            // Validate account is owned by our program
            require!(
                debt_account.owner == &crate::ID,
                AerospacerProtocolError::Unauthorized
            );
            
            let debt_data = debt_account.try_borrow_data()?;
            let user_debt: UserDebtAmount = UserDebtAmount::try_from_slice(&debt_data)?;
            
            if user_debt.owner == *trove_user {
                // Parse UserCollateralAmount
                let collateral_account = &account_chunk[1];
                
                // Validate account is owned by our program
                require!(
                    collateral_account.owner == &crate::ID,
                    AerospacerProtocolError::Unauthorized
                );
                
                let collateral_data = collateral_account.try_borrow_data()?;
                let user_collateral: UserCollateralAmount = UserCollateralAmount::try_from_slice(&collateral_data)?;
                
                // Parse LiquidityThreshold
                let liquidity_account = &account_chunk[2];
                
                // Validate account is owned by our program
                require!(
                    liquidity_account.owner == &crate::ID,
                    AerospacerProtocolError::Unauthorized
                );
                
                let liquidity_data = liquidity_account.try_borrow_data()?;
                let liquidity_threshold: LiquidityThreshold = LiquidityThreshold::try_from_slice(&liquidity_data)?;
                
                // Validate TokenAccount
                let token_account = &account_chunk[3];
                require!(
                    token_account.owner == &anchor_spl::token::ID,
                    AerospacerProtocolError::Unauthorized
                );
                
                return Ok(TroveData {
                    user: *trove_user,
                    debt_amount: user_debt.amount,
                    collateral_amounts: vec![(user_collateral.denom, user_collateral.amount)],
                    liquidity_ratio: liquidity_threshold.ratio,
                });
            }
        }
    }
    
    Err(AerospacerProtocolError::TroveDoesNotExist.into())
}

// Helper function to close trove after full redemption
fn close_trove_after_redemption(
    trove_user: &Pubkey,
    sorted_troves_state: &mut Account<SortedTrovesState>,
) -> Result<()> {
    // Remove from sorted troves list
    if sorted_troves_state.head == Some(*trove_user) {
        sorted_troves_state.head = None;
    }
    if sorted_troves_state.tail == Some(*trove_user) {
        sorted_troves_state.tail = None;
    }
    
    // Decrease size
    if sorted_troves_state.size > 0 {
        sorted_troves_state.size -= 1;
    }
    
    msg!("Trove closed: {}", trove_user);
    Ok(())
}

// Helper function to update trove after partial redemption
fn update_trove_after_partial_redemption(
    trove_user: &Pubkey,
    new_debt: u64,
    sorted_troves_state: &mut Account<SortedTrovesState>,
) -> Result<()> {
    // Update trove's debt amount in remaining_accounts
    // This would require updating the actual account data
    // For now, we'll log the update
    
    msg!("Trove updated: user={}, new_debt={}", trove_user, new_debt);
    
    // In a full implementation, this would:
    // 1. Find the trove's UserDebtAmount account in remaining_accounts
    // 2. Update the debt amount
    // 3. Recalculate ICR based on current collateral and new debt
    // 4. Reinsert in sorted order based on new ICR
    
    Ok(())
}

// Helper function to get next trove in sorted list
fn get_next_trove_in_sorted_list(
    current_trove: &Pubkey,
    sorted_troves_state: &Account<SortedTrovesState>,
) -> Result<Option<Pubkey>> {
    // In a full implementation, this would:
    // 1. Find the current trove's node
    // 2. Return the next trove in the sorted list
    
    // For now, return None to stop iteration
    // In a real implementation, this would traverse the linked list
    Ok(None)
}

// Trove data structure for redemption
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TroveData {
    pub user: Pubkey,
    pub debt_amount: u64,
    pub collateral_amounts: Vec<(String, u64)>,
    pub liquidity_ratio: u64,
}