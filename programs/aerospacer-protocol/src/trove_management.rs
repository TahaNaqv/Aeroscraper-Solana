use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::oracle::*;
use crate::account_management::*;

/// Trove management utilities
/// This module provides clean, type-safe trove operations

/// Trove operation result
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TroveOperationResult {
    pub success: bool,
    pub new_debt_amount: u64,
    pub new_collateral_amount: u64,
    pub new_icr: u64,
    pub message: String,
}

/// Liquidation operation result
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidationResult {
    pub liquidated_count: u32,
    pub total_debt_liquidated: u64,
    pub total_collateral_gained: u64,
    pub liquidation_gains: Vec<(String, u64)>, // Changed from HashMap to Vec for Anchor compatibility
}

/// Redemption operation result
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RedeemResult {
    pub collateral_sent: Vec<(String, u64)>, // Changed from HashMap to Vec for Anchor compatibility
    pub stablecoin_burned: u64,
    pub troves_redeemed: Vec<Pubkey>,
}

/// Trove manager for handling all trove operations
pub struct TroveManager;

impl TroveManager {
    /// Open a new trove
    pub fn open_trove(
        trove_ctx: &mut TroveContext,
        collateral_ctx: &mut CollateralContext,
        oracle_ctx: &OracleContext,
        loan_amount: u64,
        collateral_amount: u64,
        collateral_denom: String,
    ) -> Result<TroveOperationResult> {
        // Validate minimum amounts
        require!(
            loan_amount >= MINIMUM_LOAN_AMOUNT,
            AerospacerProtocolError::LoanAmountBelowMinimum
        );
        
        require!(
            collateral_amount >= MINIMUM_COLLATERAL_AMOUNT,
            AerospacerProtocolError::CollateralBelowMinimum
        );
        
        // Get collateral price
        let price_data = oracle_ctx.get_price(&collateral_denom)?;
        oracle_ctx.validate_price(&price_data)?;
        
        // Calculate collateral value
        let collateral_value = PriceCalculator::calculate_collateral_value(
            collateral_amount,
            price_data.price as u64, // Convert i64 to u64
            price_data.decimal,
        )?;
        
        // Calculate ICR
        let icr = PriceCalculator::calculate_collateral_ratio(
            collateral_value,
            loan_amount,
        )?;
        
        // Check minimum collateral ratio (both are simple percentages)
        let minimum_ratio = trove_ctx.state.minimum_collateral_ratio as u64;
        require!(
            icr >= minimum_ratio,
            AerospacerProtocolError::CollateralBelowMinimum
        );
        
        // Update accounts
        trove_ctx.update_debt_amount(loan_amount)?;
        trove_ctx.update_liquidity_threshold(icr)?;
        collateral_ctx.update_collateral_amount(collateral_amount)?;
        
        // Update state
        trove_ctx.state.total_debt_amount = trove_ctx.state.total_debt_amount
            .checked_add(loan_amount)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        // Transfer collateral to protocol
        collateral_ctx.transfer_to_protocol(collateral_amount)?;
        
        // Note: Sorted list insertion happens in instruction handler via sorted_troves_simple::insert_trove
        // which requires Node account access not available at this layer
        
        Ok(TroveOperationResult {
            success: true,
            new_debt_amount: loan_amount,
            new_collateral_amount: collateral_amount,
            new_icr: icr,
            message: "Trove opened successfully".to_string(),
        })
    }
    
    /// Add collateral to existing trove
    pub fn add_collateral(
        trove_ctx: &mut TroveContext,
        collateral_ctx: &mut CollateralContext,
        oracle_ctx: &OracleContext,
        additional_amount: u64,
        collateral_denom: String,
    ) -> Result<TroveOperationResult> {
        // Get current trove info
        let trove_info = trove_ctx.get_trove_info()?;
        let collateral_info = collateral_ctx.get_collateral_info()?;
        
        // Get collateral price
        let price_data = oracle_ctx.get_price(&collateral_denom)?;
        oracle_ctx.validate_price(&price_data)?;
        
        // Calculate new collateral amount
        let new_collateral_amount = collateral_info.amount
            .checked_add(additional_amount)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        // Calculate new collateral value
        let new_collateral_value = PriceCalculator::calculate_collateral_value(
            new_collateral_amount,
            price_data.price as u64, // Convert i64 to u64
            price_data.decimal,
        )?;
        
        // Calculate new ICR
        let new_icr = PriceCalculator::calculate_collateral_ratio(
            new_collateral_value,
            trove_info.debt_amount,
        )?;
        
        // Check minimum collateral ratio (both are simple percentages)
        let minimum_ratio = trove_ctx.state.minimum_collateral_ratio as u64;
        require!(
            new_icr >= minimum_ratio,
            AerospacerProtocolError::CollateralBelowMinimum
        );
        
        // Update accounts
        collateral_ctx.update_collateral_amount(new_collateral_amount)?;
        trove_ctx.update_liquidity_threshold(new_icr)?;
        
        // Transfer collateral to protocol
        collateral_ctx.transfer_to_protocol(additional_amount)?;
        
        // Note: Sorted list operations happen in instruction handler via sorted_troves_simple
        
        Ok(TroveOperationResult {
            success: true,
            new_debt_amount: trove_info.debt_amount,
            new_collateral_amount: new_collateral_amount,
            new_icr: new_icr,
            message: "Collateral added successfully".to_string(),
        })
    }
    
    /// Remove collateral from existing trove
    pub fn remove_collateral(
        trove_ctx: &mut TroveContext,
        collateral_ctx: &mut CollateralContext,
        oracle_ctx: &OracleContext,
        remove_amount: u64,
        collateral_denom: String,
    ) -> Result<TroveOperationResult> {
        // Get current trove info
        let trove_info = trove_ctx.get_trove_info()?;
        let collateral_info = collateral_ctx.get_collateral_info()?;
        
        // Validate removal amount
        require!(
            remove_amount <= collateral_info.amount,
            AerospacerProtocolError::InvalidAmount
        );
        
        // Get collateral price
        let price_data = oracle_ctx.get_price(&collateral_denom)?;
        oracle_ctx.validate_price(&price_data)?;
        
        // Calculate new collateral amount
        let new_collateral_amount = collateral_info.amount
            .checked_sub(remove_amount)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        // Check minimum collateral amount
        require!(
            new_collateral_amount >= MINIMUM_COLLATERAL_AMOUNT,
            AerospacerProtocolError::CollateralBelowMinimum
        );
        
        // Calculate new collateral value
        let new_collateral_value = PriceCalculator::calculate_collateral_value(
            new_collateral_amount,
            price_data.price as u64, // Convert i64 to u64
            price_data.decimal,
        )?;
        
        // Calculate new ICR
        let new_icr = PriceCalculator::calculate_collateral_ratio(
            new_collateral_value,
            trove_info.debt_amount,
        )?;
        
        // Check minimum collateral ratio (both are simple percentages)
        let minimum_ratio = trove_ctx.state.minimum_collateral_ratio as u64;
        require!(
            new_icr >= minimum_ratio,
            AerospacerProtocolError::CollateralBelowMinimum
        );
        
        // Update accounts
        collateral_ctx.update_collateral_amount(new_collateral_amount)?;
        trove_ctx.update_liquidity_threshold(new_icr)?;
        
        // Transfer collateral back to user
        collateral_ctx.transfer_to_user(remove_amount)?;
        
        // Note: Sorted list operations happen in instruction handler via sorted_troves_simple
        
        Ok(TroveOperationResult {
            success: true,
            new_debt_amount: trove_info.debt_amount,
            new_collateral_amount: new_collateral_amount,
            new_icr: new_icr,
            message: "Collateral removed successfully".to_string(),
        })
    }
    
    /// Borrow additional loan from existing trove
    pub fn borrow_loan(
        trove_ctx: &mut TroveContext,
        collateral_ctx: &mut CollateralContext,
        oracle_ctx: &OracleContext,
        additional_loan_amount: u64,
    ) -> Result<TroveOperationResult> {
        // Get current trove info
        let trove_info = trove_ctx.get_trove_info()?;
        let collateral_info = collateral_ctx.get_collateral_info()?;
        
        // Calculate new debt amount
        let new_debt_amount = trove_info.debt_amount
            .checked_add(additional_loan_amount)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        // Get collateral price
        let price_data = oracle_ctx.get_price(&collateral_info.denom)?;
        oracle_ctx.validate_price(&price_data)?;
        
        // Calculate collateral value
        let collateral_value = PriceCalculator::calculate_collateral_value(
            collateral_info.amount,
            price_data.price as u64, // Convert i64 to u64
            price_data.decimal,
        )?;
        
        // Calculate new ICR
        let new_icr = PriceCalculator::calculate_collateral_ratio(
            collateral_value,
            new_debt_amount,
        )?;
        
        // Check minimum collateral ratio
        let minimum_ratio = trove_ctx.state.minimum_collateral_ratio as u64 * DECIMAL_FRACTION_18 as u64;
        require!(
            new_icr >= minimum_ratio,
            AerospacerProtocolError::CollateralBelowMinimum
        );
        
        // Update accounts
        trove_ctx.update_debt_amount(new_debt_amount)?;
        trove_ctx.update_liquidity_threshold(new_icr)?;
        
        // Update state
        trove_ctx.state.total_debt_amount = trove_ctx.state.total_debt_amount
            .checked_add(additional_loan_amount)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        // Note: Sorted list operations happen in instruction handler via sorted_troves_simple
        
        Ok(TroveOperationResult {
            success: true,
            new_debt_amount: new_debt_amount,
            new_collateral_amount: collateral_info.amount,
            new_icr: new_icr,
            message: "Loan borrowed successfully".to_string(),
        })
    }
    
    /// Repay loan
    pub fn repay_loan(
        trove_ctx: &mut TroveContext,
        collateral_ctx: &mut CollateralContext,
        oracle_ctx: &OracleContext,
        repay_amount: u64,
    ) -> Result<TroveOperationResult> {
        // Get current trove info
        let trove_info = trove_ctx.get_trove_info()?;
        let collateral_info = collateral_ctx.get_collateral_info()?;
        
        // Validate repayment amount
        require!(
            repay_amount <= trove_info.debt_amount,
            AerospacerProtocolError::InvalidAmount
        );
        
        // Calculate new debt amount
        let new_debt_amount = trove_info.debt_amount
            .checked_sub(repay_amount)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        // Update state
        trove_ctx.state.total_debt_amount = trove_ctx.state.total_debt_amount
            .checked_sub(repay_amount)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        if new_debt_amount == 0 {
            // Full repayment - close trove
            trove_ctx.update_debt_amount(0)?;
            trove_ctx.update_liquidity_threshold(0)?;
            collateral_ctx.update_collateral_amount(0)?;
            
            // Return collateral to user
            collateral_ctx.transfer_to_user(collateral_info.amount)?;
            
            // Note: Sorted list operations happen in instruction handler via sorted_troves_simple
            
            Ok(TroveOperationResult {
                success: true,
                new_debt_amount: 0,
                new_collateral_amount: 0,
                new_icr: 0,
                message: "Trove fully repaid and closed".to_string(),
            })
        } else {
            // Partial repayment
            // Get collateral price for ICR calculation
            let price_data = oracle_ctx.get_price(&collateral_info.denom)?;
            oracle_ctx.validate_price(&price_data)?;
            
            // Calculate collateral value
            let collateral_value = PriceCalculator::calculate_collateral_value(
                collateral_info.amount,
                price_data.price as u64, // Convert i64 to u64
                price_data.decimal,
            )?;
            
            // Calculate new ICR
            let new_icr = PriceCalculator::calculate_collateral_ratio(
                collateral_value,
                new_debt_amount,
            )?;
            
            // Update accounts
            trove_ctx.update_debt_amount(new_debt_amount)?;
            trove_ctx.update_liquidity_threshold(new_icr)?;
            
            // Note: Sorted list operations happen in instruction handler via sorted_troves_simple
            
            Ok(TroveOperationResult {
                success: true,
                new_debt_amount: new_debt_amount,
                new_collateral_amount: collateral_info.amount,
                new_icr: new_icr,
                message: "Partial repayment successful".to_string(),
            })
        }
    }
    
    /// Liquidate undercollateralized troves
    pub fn liquidate_troves(
        liquidation_ctx: &mut LiquidationContext,
        oracle_ctx: &OracleContext,
        liquidation_list: Vec<Pubkey>,
        remaining_accounts: &[AccountInfo],
    ) -> Result<LiquidationResult> {
        let mut liquidated_count = 0u32;
        let mut total_debt_liquidated = 0u64;
        let mut total_collateral_gained = 0u64;
        let mut liquidation_gains = Vec::new();
        
        // Process each trove in the liquidation list
        for (i, user) in liquidation_list.iter().enumerate() {
            // Parse real trove data from remaining accounts
            let trove_data = parse_trove_data(user, i, remaining_accounts)?;
            
            // Validate trove is actually undercollateralized
            validate_trove_for_liquidation(&trove_data, oracle_ctx)?;
            
            // Calculate liquidation gains
            let mut trove_collateral_gain = 0u64;
            for (denom, amount) in &trove_data.collateral_amounts {
                trove_collateral_gain = trove_collateral_gain.saturating_add(*amount);
                
                // Find existing entry or add new one
                if let Some(existing) = liquidation_gains.iter_mut().find(|(d, _)| d == denom) {
                    existing.1 += *amount;
                } else {
                    liquidation_gains.push((denom.clone(), *amount));
                }
            }
            
            // Process liquidation
            liquidation_ctx.liquidate_trove(*user, trove_data.debt_amount, trove_data.collateral_amounts.clone())?;
            
            // Distribute seized collateral to stability pool stakers
            distribute_liquidation_gains_to_stakers(
                &liquidation_ctx.state,
                &trove_data.collateral_amounts,
                trove_data.debt_amount,
                remaining_accounts,
                liquidation_list.len(),
            )?;
            
            // Update user accounts to zero (trove is closed)
            update_user_accounts_after_liquidation(user, i, remaining_accounts)?;
            
            // Update counters
            liquidated_count += 1;
            total_debt_liquidated = total_debt_liquidated.saturating_add(trove_data.debt_amount);
            total_collateral_gained = total_collateral_gained.saturating_add(trove_collateral_gain);
            
            // Note: Sorted list operations happen in instruction handler via sorted_troves_simple
            
            msg!("Liquidated trove: user={}, debt={}, collateral={}", 
                 user, trove_data.debt_amount, trove_collateral_gain);
        }
        
        Ok(LiquidationResult {
            liquidated_count,
            total_debt_liquidated,
            total_collateral_gained,
            liquidation_gains,
        })
    }
    
    /// Redeem stablecoin for collateral from riskiest troves
    pub fn redeem(
        trove_ctx: &mut TroveContext,
        collateral_ctx: &mut CollateralContext,
        oracle_ctx: &OracleContext,
        redeem_amount: u64,
    ) -> Result<RedeemResult> {
        let mut collateral_sent = Vec::new();
        let mut troves_redeemed = Vec::new();
        let mut remaining_amount = redeem_amount;
        
        // Start from the riskiest trove (head of sorted list)
        // Note: In production, this would iterate through the sorted troves list
        let mut current_trove = None; // Simplified for now - full implementation needs sorted list access
        
        while let Some(trove_user) = current_trove {
            if remaining_amount == 0 {
                break;
            }
            
            // Get trove information (mock implementation for now)
            let trove_debt = 1000u64; // Mock debt amount
            let trove_collateral = vec![("SOL".to_string(), 500u64)]; // Mock collateral
            
            // Calculate how much to redeem from this trove
            let redeem_from_trove = remaining_amount.min(trove_debt);
            
            // Calculate collateral to send (proportional to debt redeemed)
            let collateral_ratio = redeem_from_trove as f64 / trove_debt as f64;
            
            for (denom, amount) in &trove_collateral {
                let collateral_to_send = ((*amount as f64) * collateral_ratio) as u64;
                
                // Find existing entry or add new one
                if let Some(existing) = collateral_sent.iter_mut().find(|(d, _)| d == denom) {
                    existing.1 += collateral_to_send;
                } else {
                    collateral_sent.push((denom.clone(), collateral_to_send));
                }
            }
            
            // Update trove debt
            let new_debt = trove_debt - redeem_from_trove;
            
            if new_debt == 0 {
                // Full redemption - close trove
                // Note: Sorted list operations happen in instruction handler via sorted_troves_simple
                msg!("Trove fully redeemed and closed: {}", trove_user);
            } else {
                // Partial redemption - update ICR
                // For now, just log - in real implementation would recalculate ICR
                msg!("Trove partially redeemed: user={}, new_debt={}", trove_user, new_debt);
            }
            
            troves_redeemed.push(trove_user);
            remaining_amount = remaining_amount.saturating_sub(redeem_from_trove);
            
            // Move to next trove (mock implementation)
            current_trove = None; // Simplified for now
        }
        
        // Update global state
        trove_ctx.state.total_debt_amount = trove_ctx.state.total_debt_amount
            .saturating_sub(redeem_amount - remaining_amount);
        
        Ok(RedeemResult {
            collateral_sent,
            stablecoin_burned: redeem_amount - remaining_amount,
            troves_redeemed,
        })
    }
}

/// Trove data structure for liquidation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TroveData {
    pub user: Pubkey,
    pub debt_amount: u64,
    pub collateral_amounts: Vec<(String, u64)>,
    pub liquidity_ratio: u64,
}

/// Parse trove data from remaining accounts
fn parse_trove_data(
    user: &Pubkey,
    user_index: usize,
    remaining_accounts: &[AccountInfo],
) -> Result<TroveData> {
    let account_start = user_index * 4; // 4 accounts per user
    
    // Validate we have enough accounts
    require!(
        account_start + 3 < remaining_accounts.len(),
        AerospacerProtocolError::InvalidList
    );
    
    // Parse UserDebtAmount account
    let debt_account = &remaining_accounts[account_start];
    let debt_amount = parse_user_debt_amount(debt_account, user)?;
    
    // Parse UserCollateralAmount account
    let collateral_account = &remaining_accounts[account_start + 1];
    let collateral_amounts = parse_user_collateral_amount(collateral_account, user)?;
    
    // Parse LiquidityThreshold account
    let liquidity_account = &remaining_accounts[account_start + 2];
    let liquidity_ratio = parse_liquidity_threshold(liquidity_account, user)?;
    
    // Parse TokenAccount (for validation)
    let token_account = &remaining_accounts[account_start + 3];
    validate_token_account(token_account, user)?;
    
    Ok(TroveData {
        user: *user,
        debt_amount,
        collateral_amounts,
        liquidity_ratio,
    })
}

/// Parse UserDebtAmount from account info
fn parse_user_debt_amount(account_info: &AccountInfo, expected_user: &Pubkey) -> Result<u64> {
    // Validate account is owned by our program
    require!(
        account_info.owner == &crate::ID,
        AerospacerProtocolError::Unauthorized
    );
    
    // Validate account is mutable
    require!(
        account_info.is_writable,
        AerospacerProtocolError::Unauthorized
    );
    
    // Parse account data
    let account_data = account_info.try_borrow_data()?;
    let user_debt_amount = UserDebtAmount::try_from_slice(&account_data)?;
    
    // Validate ownership
    require!(
        user_debt_amount.owner == *expected_user,
        AerospacerProtocolError::Unauthorized
    );
    
    Ok(user_debt_amount.amount)
}

/// Parse UserCollateralAmount from account info
fn parse_user_collateral_amount(account_info: &AccountInfo, expected_user: &Pubkey) -> Result<Vec<(String, u64)>> {
    // Validate account is owned by our program
    require!(
        account_info.owner == &crate::ID,
        AerospacerProtocolError::Unauthorized
    );
    
    // Validate account is mutable
    require!(
        account_info.is_writable,
        AerospacerProtocolError::Unauthorized
    );
    
    // Parse account data
    let account_data = account_info.try_borrow_data()?;
    let user_collateral_amount = UserCollateralAmount::try_from_slice(&account_data)?;
    
    // Validate ownership
    require!(
        user_collateral_amount.owner == *expected_user,
        AerospacerProtocolError::Unauthorized
    );
    
    Ok(vec![(user_collateral_amount.denom, user_collateral_amount.amount)])
}

/// Parse LiquidityThreshold from account info
fn parse_liquidity_threshold(account_info: &AccountInfo, expected_user: &Pubkey) -> Result<u64> {
    // Validate account is owned by our program
    require!(
        account_info.owner == &crate::ID,
        AerospacerProtocolError::Unauthorized
    );
    
    // Validate account is mutable
    require!(
        account_info.is_writable,
        AerospacerProtocolError::Unauthorized
    );
    
    // Parse account data
    let account_data = account_info.try_borrow_data()?;
    let liquidity_threshold = LiquidityThreshold::try_from_slice(&account_data)?;
    
    // Validate ownership
    require!(
        liquidity_threshold.owner == *expected_user,
        AerospacerProtocolError::Unauthorized
    );
    
    Ok(liquidity_threshold.ratio)
}

/// Validate TokenAccount
fn validate_token_account(account_info: &AccountInfo, _expected_user: &Pubkey) -> Result<()> {
    // Validate account is owned by token program
    require!(
        account_info.owner == &anchor_spl::token::ID,
        AerospacerProtocolError::Unauthorized
    );
    
    Ok(())
}

/// Validate that a trove is actually undercollateralized and can be liquidated
fn validate_trove_for_liquidation(trove_data: &TroveData, oracle_ctx: &OracleContext) -> Result<()> {
    // Calculate current collateral value
    let mut total_collateral_value = 0u64;
    
    for (denom, amount) in &trove_data.collateral_amounts {
        let price_data = oracle_ctx.get_price(denom)?;
        let collateral_value = PriceCalculator::calculate_collateral_value(
            *amount,
            price_data.price as u64,
            price_data.decimal,
        )?;
        total_collateral_value = total_collateral_value.saturating_add(collateral_value);
    }
    
    // Calculate current ICR
    let current_icr = PriceCalculator::calculate_collateral_ratio(
        total_collateral_value,
        trove_data.debt_amount,
    )?;
    
    // Check if trove is undercollateralized (ICR < 110%)
    // Both current_icr and threshold are simple percentages
    let liquidation_threshold = 110u64; // 110%
    require!(
        current_icr < liquidation_threshold,
        AerospacerProtocolError::CollateralBelowMinimum // Reuse error for now
    );
    
    msg!("Trove validated for liquidation: ICR={}, threshold={}", 
         current_icr, liquidation_threshold);
    
    Ok(())
}

/// Update user accounts after liquidation (set to zero)
fn update_user_accounts_after_liquidation(
    user: &Pubkey,
    user_index: usize,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    let account_start = user_index * 4;
    
    // Update UserDebtAmount to zero
    let debt_account = &remaining_accounts[account_start];
    let mut debt_data = debt_account.try_borrow_mut_data()?;
    let mut user_debt_amount = UserDebtAmount::try_from_slice(&debt_data)?;
    user_debt_amount.amount = 0;
    user_debt_amount.serialize(&mut &mut debt_data[..])?;
    
    // Update UserCollateralAmount to zero
    let collateral_account = &remaining_accounts[account_start + 1];
    let mut collateral_data = collateral_account.try_borrow_mut_data()?;
    let mut user_collateral_amount = UserCollateralAmount::try_from_slice(&collateral_data)?;
    user_collateral_amount.amount = 0;
    user_collateral_amount.serialize(&mut &mut collateral_data[..])?;
    
    // Update LiquidityThreshold to zero
    let liquidity_account = &remaining_accounts[account_start + 2];
    let mut liquidity_data = liquidity_account.try_borrow_mut_data()?;
    let mut liquidity_threshold = LiquidityThreshold::try_from_slice(&liquidity_data)?;
    liquidity_threshold.ratio = 0;
    liquidity_threshold.serialize(&mut &mut liquidity_data[..])?;
    
    msg!("Updated user accounts after liquidation: user={}", user);
    
    Ok(())
}

/// Distribute liquidation gains to stability pool stakers
/// 
/// This function creates or updates TotalLiquidationCollateralGain PDAs to track
/// seized collateral for distribution to stability pool stakers.
/// The actual per-user distribution is "lazy" - it happens when users call withdraw_liquidation_gains.
/// 
/// # Arguments
/// * `state` - The protocol state containing total_stake_amount
/// * `collateral_amounts` - Vector of (denom, amount) pairs seized from liquidation
/// * `debt_amount` - The debt amount that was liquidated
/// * `remaining_accounts` - Must include TotalLiquidationCollateralGain PDAs after the 4n trove accounts
/// * `num_troves` - Number of troves being liquidated (to calculate where gain PDAs start)
fn distribute_liquidation_gains_to_stakers(
    state: &StateAccount,
    collateral_amounts: &Vec<(String, u64)>,
    debt_amount: u64,
    remaining_accounts: &[AccountInfo],
    num_troves: usize,
) -> Result<()> {
    let total_stake = state.total_stake_amount;
    
    // Get current block height for tracking this liquidation event
    let current_block_height = Clock::get()?.slot;
    
    msg!("Distributing liquidation gains to stability pool:");
    msg!("  Total stake in pool: {}", total_stake);
    msg!("  Debt liquidated: {}", debt_amount);
    msg!("  Block height: {}", current_block_height);
    
    // TotalLiquidationCollateralGain PDAs start after the trove accounts (4 per trove)
    let gain_pdas_start = num_troves * 4;
    
    // Track liquidation gains in TotalLiquidationCollateralGain PDAs
    for (denom, amount) in collateral_amounts {
        // Expected PDA for this denom at current block height
        let gain_seeds = [
            b"total_liq_gain",
            &current_block_height.to_le_bytes()[..],
            denom.as_bytes(),
        ];
        let (expected_pda, _bump) = Pubkey::find_program_address(&gain_seeds, &crate::ID);
        
        // Look for the PDA in remaining_accounts starting after trove accounts
        let mut found = false;
        for i in gain_pdas_start..remaining_accounts.len() {
            let account_info = &remaining_accounts[i];
            
            if account_info.key() == expected_pda {
                // Update existing PDA
                let mut data = account_info.try_borrow_mut_data()?;
                
                if data.len() >= 8 + TotalLiquidationCollateralGain::LEN {
                    // Deserialize existing data
                    let mut gain = TotalLiquidationCollateralGain::try_deserialize(&mut &data[..])?;
                    
                    // Add to existing amount
                    gain.amount = gain.amount
                        .checked_add(*amount)
                        .ok_or(AerospacerProtocolError::OverflowError)?;
                    
                    // Serialize back
                    gain.try_serialize(&mut &mut data[..])?;
                    
                    msg!("  Updated gain PDA for {}: added {}, total now {}", 
                         denom, amount, gain.amount);
                } else {
                    // Initialize new PDA
                    let gain = TotalLiquidationCollateralGain {
                        block_height: current_block_height,
                        denom: denom.clone(),
                        amount: *amount,
                    };
                    gain.try_serialize(&mut &mut data[..])?;
                    
                    msg!("  Initialized gain PDA for {}: {}", denom, amount);
                }
                
                found = true;
                break;
            }
        }
        
        if !found {
            if total_stake == 0 {
                msg!("  {} {} seized - no stakers, gains remain in protocol vault", denom, amount);
            } else {
                msg!("  WARNING: Gain PDA not provided for {} - gains tracked in vault only", denom);
                msg!("  {} stakers can still withdraw proportionally from vault balance", total_stake);
            }
        }
    }
    
    msg!("Liquidation gains distribution complete");
    
    Ok(())
}
