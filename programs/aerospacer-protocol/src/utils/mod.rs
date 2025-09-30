pub mod calculations;
pub mod fee_processing;
pub mod price_queries;

pub use calculations::*;
pub use fee_processing::*;
pub use price_queries::*;

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::AerospacerProtocolError;

// Safe math operations
pub fn safe_add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or(AerospacerProtocolError::Overflow.into())
}

pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b).ok_or(AerospacerProtocolError::Overflow.into())
}

pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b).ok_or(AerospacerProtocolError::Overflow.into())
}

pub fn safe_div(a: u64, b: u64) -> Result<u64> {
    if b == 0 {
        return Err(AerospacerProtocolError::InvalidAmount.into());
    }
    Ok(a / b)
}

// Validation functions
pub fn validate_collateral_params(
    amount: u64,
    mint: Pubkey,
    minimum_amount: u64,
) -> Result<()> {
    if amount < minimum_amount {
        return Err(AerospacerProtocolError::CollateralBelowMinimum.into());
    }
    
    // Additional validation can be added here
    Ok(())
}

pub fn validate_trove_parameters(
    trove: &TroveAccount,
    minimum_collateral_ratio: u64,
    collateral_price: u64,
) -> Result<()> {
    let current_ratio = calculate_collateral_ratio(
        trove.collateral_amount,
        trove.debt_amount,
        collateral_price,
    )?;
    
    if current_ratio < minimum_collateral_ratio {
        return Err(AerospacerProtocolError::InsufficientCollateral.into());
    }
    
    Ok(())
}

// Per-denom PDA utility functions
pub fn get_total_collateral_pda(denom: &str, program_id: &Pubkey) -> (Pubkey, u8) {
    let seeds = TotalCollateralByDenom::pda_seeds(denom);
    Pubkey::find_program_address(&seeds, program_id)
}

pub fn get_protocol_stablecoin_vault_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    let seeds = ProtocolVault::stablecoin_vault_seeds();
    Pubkey::find_program_address(&seeds, program_id)
}

pub fn get_protocol_collateral_vault_pda(denom: &str, program_id: &Pubkey) -> (Pubkey, u8) {
    let seeds = ProtocolVault::collateral_vault_seeds(denom);
    Pubkey::find_program_address(&seeds, program_id)
}

// Helper to get total collateral amount for a denom
pub fn get_total_collateral_amount<'a>(
    denom: &str,
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> Result<u64> {
    let (pda, _bump) = get_total_collateral_pda(denom, program_id);
    
    // Find the account in remaining_accounts
    for account in accounts {
        if account.key() == pda {
            let total_collateral: Account<TotalCollateralByDenom> = 
                Account::try_from(account)?;
            return Ok(total_collateral.total_amount);
        }
    }
    
    // If not found, return 0 (denom not yet initialized)
    Ok(0)
}

// Helper to update total collateral amount for a denom
pub fn update_total_collateral_amount<'a>(
    denom: &str,
    amount_change: i64, // positive for increase, negative for decrease
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> Result<()> {
    let (pda, _bump) = get_total_collateral_pda(denom, program_id);
    
    // Find the account in remaining_accounts
    for account in accounts {
        if account.key() == pda {
            let mut total_collateral: Account<TotalCollateralByDenom> = 
                Account::try_from(account)?;
            
            if amount_change > 0 {
                total_collateral.total_amount = safe_add(
                    total_collateral.total_amount,
                    amount_change as u64,
                )?;
            } else {
                total_collateral.total_amount = safe_sub(
                    total_collateral.total_amount,
                    (-amount_change) as u64,
                )?;
            }
            
            total_collateral.last_updated = Clock::get()?.unix_timestamp;
            return Ok(());
        }
    }
    
    Err(AerospacerProtocolError::AccountNotFound.into())
}

// Helper to update total collateral amount from AccountInfo directly
pub fn update_total_collateral_from_account_info<'a>(
    account_info: &'a AccountInfo<'a>,
    amount_change: i64, // positive for increase, negative for decrease
) -> Result<()> {
    if account_info.data_is_empty() {
        return Err(AerospacerProtocolError::AccountNotFound.into());
    }
    
    let mut total_collateral: Account<TotalCollateralByDenom> = 
        Account::try_from(account_info)?;
    
    if amount_change > 0 {
        total_collateral.total_amount = safe_add(
            total_collateral.total_amount,
            amount_change as u64,
        )?;
    } else {
        total_collateral.total_amount = safe_sub(
            total_collateral.total_amount,
            (-amount_change) as u64,
        )?;
    }
    
    total_collateral.last_updated = Clock::get()?.unix_timestamp;
    
    let mut data = account_info.try_borrow_mut_data()?;
    total_collateral.try_serialize(&mut *data)?;
    
    Ok(())
}

// Constants moved to crate::state to avoid ambiguity
