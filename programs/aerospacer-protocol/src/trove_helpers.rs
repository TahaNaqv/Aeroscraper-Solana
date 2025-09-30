use anchor_lang::prelude::*;
use crate::state::*;
use crate::utils::*;

/// Trove helper functions
pub struct TroveHelpers;

impl TroveHelpers {
    /// Create a new trove
    pub fn create_trove(
        owner: Pubkey,
        collateral_amount: u64,
        debt_amount: u64,
        collateral_price: u64,
        minimum_collateral_ratio: u64,
        collateral_denom: String,
    ) -> Result<TroveAccount> {
        // Validate parameters
        let temp = TroveAccount {
            owner,
            collateral_amount,
            debt_amount,
            collateral_ratio: 0,
            created_at: Clock::get()?.unix_timestamp,
            collateral_denom: collateral_denom.clone(),
            is_active: true,
        };
        validate_trove_parameters(
            &temp,
            minimum_collateral_ratio,
            collateral_price,
        )?;
        
        let trove = TroveAccount {
            owner,
            collateral_amount,
            debt_amount,
            collateral_ratio: calculate_collateral_ratio(collateral_amount, debt_amount, collateral_price)?,
            created_at: Clock::get()?.unix_timestamp,
            collateral_denom,
            is_active: true,
        };
        
        Ok(trove)
    }
    
    /// Update trove collateral
    pub fn update_collateral(
        trove: &mut TroveAccount,
        new_collateral_amount: u64,
        collateral_price: u64,
        minimum_collateral_ratio: u64,
    ) -> Result<()> {
        trove.collateral_amount = new_collateral_amount;
        trove.collateral_ratio = calculate_collateral_ratio(
            trove.collateral_amount,
            trove.debt_amount,
            collateral_price,
        )?;
        
        // Validate the updated trove
        validate_trove_parameters(
            trove,
            minimum_collateral_ratio,
            collateral_price,
        )?;
        
        Ok(())
    }
    
    /// Update trove debt
    pub fn update_debt(
        trove: &mut TroveAccount,
        new_debt_amount: u64,
        collateral_price: u64,
        minimum_collateral_ratio: u64,
    ) -> Result<()> {
        trove.debt_amount = new_debt_amount;
        trove.collateral_ratio = calculate_collateral_ratio(
            trove.collateral_amount,
            trove.debt_amount,
            collateral_price,
        )?;
        
        // Validate the updated trove
        validate_trove_parameters(
            trove,
            minimum_collateral_ratio,
            collateral_price,
        )?;
        
        Ok(())
    }
    
    /// Check if trove is active
    pub fn is_active(trove: &TroveAccount) -> bool {
        trove.is_active && (trove.collateral_amount > 0 || trove.debt_amount > 0)
    }
    
    /// Calculate trove net value
    pub fn calculate_net_value(
        trove: &TroveAccount,
        collateral_price: u64,
    ) -> Result<i64> {
        let collateral_value = trove.collateral_amount
            .checked_mul(collateral_price)
            .ok_or(ErrorCode::Overflow)?;
        
        let debt_value = trove.debt_amount as u64; // Assuming 1:1 for stablecoin
        
        let net_value = collateral_value as i64 - debt_value as i64;
        Ok(net_value)
    }
    
    /// Check if trove can be liquidated
    pub fn can_be_liquidated(
        trove: &TroveAccount,
        collateral_price: u64,
        minimum_collateral_ratio: u64,
    ) -> Result<bool> {
        can_liquidate_trove(
            trove.collateral_amount,
            trove.debt_amount,
            collateral_price,
            minimum_collateral_ratio,
        )
    }
    
    /// Calculate liquidation reward
    pub fn calculate_liquidation_reward(
        trove: &TroveAccount,
        collateral_price: u64,
    ) -> Result<u64> {
        calculate_liquidation_reward(
            trove.debt_amount,
            trove.collateral_amount,
            collateral_price,
        )
    }
    
    /// Check trove ICR for liquidation
    pub fn check_trove_icr(
        trove: &TroveAccount,
        collateral_price: u64,
        minimum_collateral_ratio: u64,
    ) -> Result<bool> {
        let ratio = Self::get_trove_icr(trove, collateral_price)?;
        Self::check_trove_icr_with_ratio(ratio, minimum_collateral_ratio)
    }
    
    /// Get trove denoms
    pub fn get_trove_denoms(trove: &TroveAccount) -> Vec<String> {
        vec![trove.collateral_denom.clone()]
    }
    
    /// Get trove liquidity ratios
    pub fn get_trove_liquidity_ratios(
        trove: &TroveAccount,
        collateral_price: u64,
    ) -> Result<std::collections::HashMap<String, u64>> {
        let mut ratios = std::collections::HashMap::new();
        
        if trove.collateral_amount > 0 {
            let ratio = calculate_collateral_ratio(
                trove.collateral_amount,
                trove.debt_amount,
                collateral_price,
            )?;
            ratios.insert(trove.collateral_denom.clone(), ratio);
        }
        
        Ok(ratios)
    }
    
    /// Check trove ICR with ratio
    pub fn check_trove_icr_with_ratio(
        ratio: u64,
        minimum_collateral_ratio: u64,
    ) -> Result<bool> {
        if ratio < minimum_collateral_ratio {
            return Err(ErrorCode::InsufficientCollateral.into());
        }
        Ok(true)
    }
    
    /// Get trove ICR
    pub fn get_trove_icr(
        trove: &TroveAccount,
        collateral_price: u64,
    ) -> Result<u64> {
        calculate_collateral_ratio(
            trove.collateral_amount,
            trove.debt_amount,
            collateral_price,
        )
    }
}

/// Check if trove can be liquidated
pub fn can_liquidate_trove(
    collateral_amount: u64,
    debt_amount: u64,
    collateral_price: u64,
    minimum_collateral_ratio: u64,
) -> Result<bool> {
    if debt_amount == 0 {
        return Ok(false);
    }
    
    let collateral_ratio = calculate_collateral_ratio(collateral_amount, debt_amount, collateral_price)?;
    Ok(collateral_ratio < minimum_collateral_ratio)
}

/// Calculate liquidation reward
pub fn calculate_liquidation_reward(
    debt_amount: u64,
    collateral_amount: u64,
    collateral_price: u64,
) -> Result<u64> {
    let debt_value = debt_amount; // Assuming 1:1 for stablecoin
    let collateral_value = collateral_amount
        .checked_mul(collateral_price)
        .ok_or(ErrorCode::Overflow)?;
    
    // Liquidation reward is the excess collateral
    if collateral_value > debt_value {
        collateral_value
            .checked_sub(debt_value)
            .ok_or(ErrorCode::Overflow.into())
    } else {
        Ok(0)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
} 