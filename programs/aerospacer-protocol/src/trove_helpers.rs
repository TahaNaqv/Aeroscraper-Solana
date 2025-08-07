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
        validate_trove_parameters(
            collateral_amount,
            debt_amount,
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
            trove.collateral_amount,
            trove.debt_amount,
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
            trove.collateral_amount,
            trove.debt_amount,
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
}

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
} 