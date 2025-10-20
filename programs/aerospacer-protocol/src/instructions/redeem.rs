use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, Burn};
use crate::state::*;
use crate::error::*;
use crate::fees_integration::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RedeemParams {
    pub amount: u64, // Equivalent to Uint256
    pub collateral_denom: String, // Which collateral to redeem (SOL, ETH, BTC, etc.)
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: RedeemParams)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub state: Box<Account<'info, StateAccount>>,

    #[account(
        mut,
        seeds = [b"user_debt_amount", user.key().as_ref()],
        bump,
        constraint = user_debt_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_debt_amount: Box<Account<'info, UserDebtAmount>>,

    #[account(
        mut,
        seeds = [b"liquidity_threshold", user.key().as_ref()],
        bump,
        constraint = liquidity_threshold.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub liquidity_threshold: Box<Account<'info, LiquidityThreshold>>,

    #[account(
        mut,
        constraint = user_stablecoin_account.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_stablecoin_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"user_collateral_amount", user.key().as_ref(), params.collateral_denom.as_bytes()],
        bump,
        constraint = user_collateral_amount.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_collateral_amount: Box<Account<'info, UserCollateralAmount>>,

    #[account(
        mut,
        constraint = user_collateral_account.owner == user.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub user_collateral_account: Box<Account<'info, TokenAccount>>,

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
        seeds = [b"protocol_collateral_vault", params.collateral_denom.as_bytes()],
        bump
    )]
    pub protocol_collateral_vault: AccountInfo<'info>,

    /// CHECK: This is the stable coin mint account
    #[account(
        mut,
        constraint = stable_coin_mint.key() == state.stable_coin_addr @ AerospacerProtocolError::InvalidMint
    )]
    pub stable_coin_mint: UncheckedAccount<'info>,

    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral_amount", params.collateral_denom.as_bytes()],
        bump
    )]
    pub total_collateral_amount: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"sorted_troves_state"],
        bump
    )]
    pub sorted_troves_state: Box<Account<'info, SortedTrovesState>>,

    // Oracle context - integration with our aerospacer-oracle
    /// CHECK: Our oracle program - validated against state
    #[account(
        mut,
        constraint = oracle_program.key() == state.oracle_helper_addr @ AerospacerProtocolError::Unauthorized
    )]
    pub oracle_program: AccountInfo<'info>,
    
    /// CHECK: Oracle state account - validated against state
    #[account(
        mut,
        constraint = oracle_state.key() == state.oracle_state_addr @ AerospacerProtocolError::Unauthorized
    )]
    pub oracle_state: AccountInfo<'info>,

    // Fee distribution accounts
    /// CHECK: Fees program - validated against state
    #[account(
        constraint = fees_program.key() == state.fee_distributor_addr @ AerospacerProtocolError::Unauthorized
    )]
    pub fees_program: AccountInfo<'info>,
    
    /// CHECK: Fees state account - validated against state
    #[account(
        mut,
        constraint = fees_state.key() == state.fee_state_addr @ AerospacerProtocolError::Unauthorized
    )]
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
    // PRODUCTION VALIDATION: Input parameter checks
    require!(
        params.amount > 0,
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        params.amount >= MINIMUM_LOAN_AMOUNT,
        AerospacerProtocolError::InvalidAmount
    );
    
    require!(
        !params.collateral_denom.is_empty(),
        AerospacerProtocolError::InvalidAmount
    );
    
    // Store protocol fee before creating mutable borrow
    let protocol_fee = ctx.accounts.state.protocol_fee;
    
    let state = &mut ctx.accounts.state;
    
    // Validate redemption amount against total system debt
    require!(
        params.amount <= state.total_debt_amount,
        AerospacerProtocolError::NotEnoughLiquidityForRedeem
    );
    
    // PRODUCTION VALIDATION: Sorted list integrity check
    // If there's debt in the system, there must be troves in the sorted list
    if state.total_debt_amount > 0 {
        require!(
            ctx.accounts.sorted_troves_state.head.is_some(),
            AerospacerProtocolError::InvalidList
        );
    }
    
    // Validate user has enough stablecoins (including fee)
    require!(
        ctx.accounts.user_stablecoin_account.amount >= params.amount,
        AerospacerProtocolError::InvalidAmount
    );
    
    // Collect redemption fee via CPI to aerospacer-fees
    // This returns the net amount after fee deduction
    let net_redemption_amount = process_protocol_fee(
        params.amount,
        protocol_fee,
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
    msg!("Redemption fee: {} aUSD ({}%)", fee_amount, protocol_fee);
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
    // Use invoke_signed for PDA authority
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
    anchor_spl::token::burn(burn_ctx, net_redemption_amount)?;

    // Implement REAL core redemption logic using NET amount (after fee)
    let mut remaining_amount = net_redemption_amount;
    let mut total_collateral_sent = 0u64;
    let mut troves_redeemed = 0u32;
    
    // PRODUCTION SAFETY: Maximum iteration limit to prevent infinite loops
    const MAX_TROVES_PER_REDEMPTION: u32 = 100;
    let mut iteration_count = 0u32;
    
    // Start from the riskiest trove (head of sorted list)
    let mut current_trove = ctx.accounts.sorted_troves_state.head;
    
    while let Some(trove_user) = current_trove {
        if remaining_amount == 0 {
            break;
        }
        
        // PRODUCTION SAFETY: Check iteration limit
        iteration_count = iteration_count.checked_add(1)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        require!(
            iteration_count <= MAX_TROVES_PER_REDEMPTION,
            AerospacerProtocolError::InvalidList
        );
        
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
        
        // Track whether this trove has the requested collateral
        let mut collateral_sent_from_trove = 0u64;
        
        // Process ONLY the specified collateral type
        for (denom, amount) in &trove_data.collateral_amounts {
            // Skip collateral types that don't match the requested denomination
            if denom != &params.collateral_denom {
                continue;
            }
            
            let collateral_to_send = ((*amount as f64) * collateral_ratio) as u64;
            
            if collateral_to_send > 0 {
                // Transfer collateral to user
                // Use invoke_signed for PDA authority
                let collateral_seeds = &[
                    b"protocol_collateral_vault".as_ref(),
                    params.collateral_denom.as_bytes(),
                    &[ctx.bumps.protocol_collateral_vault],
                ];
                let collateral_signer = &[&collateral_seeds[..]];
                
                let collateral_transfer_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.protocol_collateral_vault.to_account_info(),
                        to: ctx.accounts.user_collateral_account.to_account_info(),
                        authority: ctx.accounts.protocol_collateral_vault.to_account_info(),
                    },
                    collateral_signer,
                );
                anchor_spl::token::transfer(collateral_transfer_ctx, collateral_to_send)?;
                
                // BUG FIX: Update UserCollateralAmount to reflect decreased collateral
                // Find the collateral_account for this trove in remaining_accounts
                for account_chunk in ctx.remaining_accounts.chunks(7) {
                    if account_chunk.len() >= 7 {
                        let debt_account = &account_chunk[0];
                        if let Ok(data) = debt_account.try_borrow_data() {
                            if let Ok(user_debt) = UserDebtAmount::try_deserialize(&mut &data[..]) {
                                if user_debt.owner == trove_user {
                                    drop(data);
                                    
                                    // Found the right chunk - update collateral_account at index 1
                                    let collateral_account = &account_chunk[1];
                                    let mut collateral_data = collateral_account.try_borrow_mut_data()?;
                                    let mut user_collateral = UserCollateralAmount::try_deserialize(&mut &collateral_data[..])?;
                                    
                                    user_collateral.amount = user_collateral.amount.saturating_sub(collateral_to_send);
                                    
                                    // Serialize back to full buffer (including discriminator)
                                    user_collateral.try_serialize(&mut &mut collateral_data[..])?;
                                    
                                    msg!("Updated UserCollateralAmount for {}: {} -> {} (sent: {})", 
                                        trove_user, *amount, user_collateral.amount, collateral_to_send);
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // Update global total_collateral_amount PDA
                let mut total_coll_data = ctx.accounts.total_collateral_amount.try_borrow_mut_data()?;
                let mut total_collateral: TotalCollateralAmount = TotalCollateralAmount::try_deserialize(&mut &total_coll_data[..])?;
                total_collateral.amount = total_collateral.amount.checked_sub(collateral_to_send)
                    .ok_or(AerospacerProtocolError::OverflowError)?;
                total_collateral.try_serialize(&mut &mut total_coll_data[..])?;
                msg!("Updated global total_collateral_amount: decreased by {}", collateral_to_send);
                
                total_collateral_sent = total_collateral_sent.saturating_add(collateral_to_send);
                collateral_sent_from_trove = collateral_sent_from_trove.saturating_add(collateral_to_send);
                msg!("Transferred {} {} to user", collateral_to_send, denom);
            }
        }
        
        // Skip this trove if it doesn't have the requested collateral type
        if collateral_sent_from_trove == 0 {
            msg!("Trove {} has no {} collateral, skipping", trove_user, params.collateral_denom);
            // Get next trove without modifying current one
            current_trove = get_next_trove_in_sorted_list(&trove_user, &ctx.accounts.sorted_troves_state, &ctx.remaining_accounts)?;
            continue;
        }
        
        // CRITICAL: Get next trove BEFORE modifying/removing current trove
        // This ensures we can continue iteration even if current trove is removed
        let next_trove = get_next_trove_in_sorted_list(&trove_user, &ctx.accounts.sorted_troves_state, &ctx.remaining_accounts)?;
        
        // Update trove debt (REAL implementation)
        let new_debt = trove_data.debt_amount.saturating_sub(redeem_from_trove);
        
        if new_debt == 0 {
            // Full redemption - close trove
            close_trove_after_redemption(&trove_user, &mut ctx.accounts.sorted_troves_state, &ctx.remaining_accounts)?;
            msg!("Trove fully redeemed and closed: {}", trove_user);
        } else {
            // Partial redemption - update trove state
            update_trove_after_partial_redemption(&trove_user, new_debt, &mut ctx.accounts.sorted_troves_state, &ctx.remaining_accounts)?;
            msg!("Trove partially redeemed: user={}, new_debt={}", trove_user, new_debt);
        }
        
        troves_redeemed += 1;
        remaining_amount = remaining_amount.saturating_sub(redeem_from_trove);
        
        // Move to next trove (already captured before modifications)
        current_trove = next_trove;
    }
    
    // CRITICAL: Require that the FULL redemption amount was processed
    // Since we already burned the stablecoins upfront, we must ensure
    // sufficient collateral was found, otherwise revert the entire transaction
    require!(
        remaining_amount == 0,
        AerospacerProtocolError::InsufficientCollateral // Not enough troves with requested collateral type
    );
    
    // PRODUCTION SAFETY: Update global state with net redeemed amount (which equals net_redemption_amount since remaining is 0)
    state.total_debt_amount = state.total_debt_amount.checked_sub(net_redemption_amount)
        .ok_or(AerospacerProtocolError::OverflowError)?;
    
    msg!("Redeemed successfully");
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Gross amount: {} aUSD", params.amount);
    msg!("Fee: {} aUSD ({}%)", fee_amount, ctx.accounts.state.protocol_fee);
    msg!("Net redemption: {} aUSD", net_redemption_amount);
    msg!("Collateral sent: {} {}", total_collateral_sent, params.collateral_denom);
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
    // Each trove has 7 accounts: UserDebtAmount, UserCollateralAmount, LiquidityThreshold, TokenAccount, UserNode, PrevNode, NextNode
    for account_chunk in remaining_accounts.chunks(7) {
        if account_chunk.len() >= 7 {
            // Parse UserDebtAmount
            let debt_account = &account_chunk[0];
            
            // Validate account is owned by our program
            require!(
                debt_account.owner == &crate::ID,
                AerospacerProtocolError::Unauthorized
            );
            
            let debt_data = debt_account.try_borrow_data()?;
            let user_debt: UserDebtAmount = UserDebtAmount::try_deserialize(&mut &debt_data[..])?;
            
            if user_debt.owner == *trove_user {
                // Parse UserCollateralAmount
                let collateral_account = &account_chunk[1];
                
                // Validate account is owned by our program
                require!(
                    collateral_account.owner == &crate::ID,
                    AerospacerProtocolError::Unauthorized
                );
                
                let collateral_data = collateral_account.try_borrow_data()?;
                let user_collateral: UserCollateralAmount = UserCollateralAmount::try_deserialize(&mut &collateral_data[..])?;
                
                // Parse LiquidityThreshold
                let liquidity_account = &account_chunk[2];
                
                // Validate account is owned by our program
                require!(
                    liquidity_account.owner == &crate::ID,
                    AerospacerProtocolError::Unauthorized
                );
                
                let liquidity_data = liquidity_account.try_borrow_data()?;
                let liquidity_threshold: LiquidityThreshold = LiquidityThreshold::try_deserialize(&mut &liquidity_data[..])?;
                
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
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    // Extract Node accounts from 7-account chunk
    // Find the chunk for this trove_user first
    for chunk in remaining_accounts.chunks(7) {
        if chunk.len() >= 7 {
            // Verify this is the right trove by checking UserDebtAmount
            let debt_account = &chunk[0];
            if let Ok(data) = debt_account.try_borrow_data() {
                if let Ok(user_debt) = UserDebtAmount::try_deserialize(&mut &data[..]) {
                    if user_debt.owner == *trove_user {
                        drop(data);
                        
                        // Found the right chunk - extract Node accounts
                        let node_accounts = &chunk[4..7]; // UserNode, PrevNode, NextNode
                        
                        // Call sorted_troves_simple::remove_trove
                        use crate::sorted_troves;
                        sorted_troves::remove_trove(
                            sorted_troves_state,
                            *trove_user,
                            node_accounts,
                        )?;
                        
                        msg!("Trove fully redeemed - removed from sorted list: {}", trove_user);
                        
                        // BUG FIX: Zero out the trove's accounts to prevent orphaned state
                        // 1. Zero out debt_account (chunk[0])
                        {
                            let mut debt_data = debt_account.try_borrow_mut_data()?;
                            let mut user_debt = UserDebtAmount::try_deserialize(&mut &debt_data[..])?;
                            user_debt.amount = 0;
                            user_debt.try_serialize(&mut &mut debt_data[..])?;
                            msg!("Zeroed UserDebtAmount for {}", trove_user);
                        }
                        
                        // 2. Zero out collateral_account (chunk[1])
                        {
                            let collateral_account = &chunk[1];
                            let mut collateral_data = collateral_account.try_borrow_mut_data()?;
                            let mut user_collateral = UserCollateralAmount::try_deserialize(&mut &collateral_data[..])?;
                            user_collateral.amount = 0;
                            user_collateral.try_serialize(&mut &mut collateral_data[..])?;
                            msg!("Zeroed UserCollateralAmount for {}", trove_user);
                        }
                        
                        // 3. Zero out liquidity_account (chunk[2])
                        {
                            let liquidity_account = &chunk[2];
                            let mut liquidity_data = liquidity_account.try_borrow_mut_data()?;
                            let mut liquidity_threshold = LiquidityThreshold::try_deserialize(&mut &liquidity_data[..])?;
                            liquidity_threshold.ratio = 0;
                            liquidity_threshold.try_serialize(&mut &mut liquidity_data[..])?;
                            msg!("Zeroed LiquidityThreshold for {}", trove_user);
                        }
                        
                        return Ok(());
                    }
                }
            }
        }
    }
    
    Err(AerospacerProtocolError::TroveDoesNotExist.into())
}

// Helper function to update trove after partial redemption
fn update_trove_after_partial_redemption(
    trove_user: &Pubkey,
    new_debt: u64,
    sorted_troves_state: &mut Account<SortedTrovesState>,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    // Find and update the UserDebtAmount and LiquidityThreshold for this trove
    // Each trove has 7 accounts: UserDebtAmount, UserCollateralAmount, LiquidityThreshold, TokenAccount, UserNode, PrevNode, NextNode
    for account_chunk in remaining_accounts.chunks(7) {
        if account_chunk.len() >= 7 {
            let debt_account = &account_chunk[0];
            
            // Check if this is the right trove
            if let Ok(data) = debt_account.try_borrow_data() {
                if let Ok(user_debt) = UserDebtAmount::try_deserialize(&mut &data[..]) {
                    if user_debt.owner == *trove_user {
                        drop(data);
                        
                        // STEP 1: Update UserDebtAmount with proper mut borrow
                        {
                            let mut debt_data_mut = debt_account.try_borrow_mut_data()?;
                            let mut user_debt_mut = UserDebtAmount::try_deserialize(&mut &debt_data_mut[..])?;
                            user_debt_mut.amount = new_debt;
                            
                            // Serialize back to full buffer (including discriminator)
                            user_debt_mut.try_serialize(&mut &mut debt_data_mut[..])?;
                            
                            msg!("Updated UserDebtAmount for {}: new_debt={}", trove_user, new_debt);
                        }
                        
                        // STEP 2: Calculate new ICR from collateral and new debt
                        let collateral_account = &account_chunk[1];
                        let liquidity_account = &account_chunk[2];
                        
                        let new_icr = {
                            let coll_data = collateral_account.try_borrow_data()?;
                            let user_collateral = UserCollateralAmount::try_deserialize(&mut &coll_data[..])?;
                            
                            let collateral_value = user_collateral.amount;
                            if new_debt > 0 {
                                (collateral_value.saturating_mul(100)) / new_debt
                            } else {
                                u64::MAX
                            }
                        };
                        
                        // STEP 3: Update LiquidityThreshold with new ICR
                        let old_icr = {
                            let mut lt_data_mut = liquidity_account.try_borrow_mut_data()?;
                            let mut liquidity_threshold = LiquidityThreshold::try_deserialize(&mut &lt_data_mut[..])?;
                            let old_icr = liquidity_threshold.ratio;
                            
                            liquidity_threshold.ratio = new_icr;
                            
                            // Serialize back to full buffer (including discriminator)
                            liquidity_threshold.try_serialize(&mut &mut lt_data_mut[..])?;
                            
                            old_icr
                        };
                        
                        msg!("Partial redemption - updated ICR: {} -> {} (debt: {})", 
                            old_icr, new_icr, new_debt);
                        
                        // STEP 4: Reinsert trove in sorted list if ICR changed significantly
                        // Get the user's Node account from chunk[4]
                        let node_account = &account_chunk[4];
                        let mut node_data = node_account.try_borrow_mut_data()?;
                        let mut user_node = Node::try_deserialize(&mut &node_data[..])?;
                        
                        use crate::sorted_troves;
                        sorted_troves::reinsert_trove(
                            sorted_troves_state,
                            &mut user_node,
                            *trove_user,
                            new_icr,
                            remaining_accounts,
                        )?;
                        
                        // Serialize updated Node back
                        user_node.try_serialize(&mut &mut node_data[..])?;
                        drop(node_data);
                        
                        msg!("Trove reinserted in sorted list after partial redemption");
                        
                        return Ok(());
                    }
                }
            }
        }
    }
    
    msg!("Warning: Could not find trove {} in remaining_accounts", trove_user);
    Ok(())
}

// Helper function to get next trove in sorted list
fn get_next_trove_in_sorted_list(
    current_trove: &Pubkey,
    _sorted_troves_state: &Account<SortedTrovesState>,
    remaining_accounts: &[AccountInfo],
) -> Result<Option<Pubkey>> {
    // Find the current trove's Node account in remaining_accounts to get next_id
    // With 7-account chunks: [UserDebtAmount, UserCollateralAmount, LiquidityThreshold, TokenAccount, UserNode, PrevNode, NextNode]
    // UserNode is at index 4 within each chunk
    
    // SECURITY: Derive expected Node PDA for current_trove
    let (expected_node_address, _bump) = Pubkey::find_program_address(
        &[b"node", current_trove.as_ref()],
        &crate::ID
    );
    
    // First, try optimized search through 7-account chunks
    for chunk in remaining_accounts.chunks(7) {
        if chunk.len() >= 7 {
            // Check if UserNode (index 4) matches current_trove
            let node_account = &chunk[4];
            
            if *node_account.key == expected_node_address {
                // SECURITY: Verify account is owned by this program
                if node_account.owner != &crate::ID {
                    msg!("Error: Node PDA {} has invalid owner", node_account.key());
                    return Err(AerospacerProtocolError::Unauthorized.into());
                }
                
                // Deserialize Node to get next_id
                if let Ok(data) = node_account.try_borrow_data() {
                    if data.len() < 8 {
                        msg!("Error: Node PDA {} has invalid size", node_account.key());
                        return Err(AerospacerProtocolError::InvalidList.into());
                    }
                    
                    if let Ok(node) = Node::try_deserialize(&mut data.as_ref()) {
                        // Verify node.id matches current_trove
                        if node.id != *current_trove {
                            msg!("Error: Node PDA has mismatched id: expected {}, found {}", current_trove, node.id);
                            return Err(AerospacerProtocolError::InvalidList.into());
                        }
                        
                        msg!("Traversing from {} to next: {:?}", current_trove, node.next_id);
                        return Ok(node.next_id);
                    }
                }
            }
        }
    }
    
    // Fallback: Search all remaining_accounts in case chunks are misaligned
    for account in remaining_accounts.iter() {
        if *account.key == expected_node_address {
            if account.owner != &crate::ID {
                msg!("Error: Node PDA {} has invalid owner", account.key());
                return Err(AerospacerProtocolError::Unauthorized.into());
            }
            
            if let Ok(data) = account.try_borrow_data() {
                if data.len() >= 8 {
                    if let Ok(node) = Node::try_deserialize(&mut data.as_ref()) {
                        if node.id == *current_trove {
                            msg!("Traversing from {} to next: {:?}", current_trove, node.next_id);
                            return Ok(node.next_id);
                        }
                    }
                }
            }
        }
    }
    
    msg!("Error: Required Node PDA {} not found in remaining_accounts", expected_node_address);
    Err(AerospacerProtocolError::InvalidList.into())
}

// Trove data structure for redemption
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TroveData {
    pub user: Pubkey,
    pub debt_amount: u64,
    pub collateral_amounts: Vec<(String, u64)>,
    pub liquidity_ratio: u64,
}