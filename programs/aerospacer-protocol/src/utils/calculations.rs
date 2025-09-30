use anchor_lang::prelude::*;

/// Calculate collateral ratio (equivalent to INJECTIVE's get_trove_icr)
pub fn calculate_collateral_ratio(
    collateral_amount: u64,
    debt_amount: u64,
    collateral_price: u64,
) -> Result<u64> {
    if debt_amount == 0 {
        return Ok(0);
    }
    
    let collateral_value = collateral_amount
        .checked_mul(collateral_price)
        .ok_or(ErrorCode::Overflow)?;
    
    let ratio = collateral_value
        .checked_mul(10000) // 100% = 10000 basis points
        .ok_or(ErrorCode::Overflow)?
        .checked_div(debt_amount)
        .ok_or(ErrorCode::Overflow)?;
    
    Ok(ratio)
}

/// Calculate protocol fee (equivalent to INJECTIVE's fee calculation)
pub fn calculate_protocol_fee(amount: u64, fee_rate: u8) -> Result<u64> {
    amount
        .checked_mul(fee_rate as u64)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(1000) // fee_rate in basis points (5% = 50)
        .ok_or(ErrorCode::Overflow.into())
}

/// Calculate net amount after protocol fee
pub fn calculate_net_amount(amount: u64, fee_rate: u8) -> Result<u64> {
    let fee_amount = calculate_protocol_fee(amount, fee_rate)?;
    amount
        .checked_sub(fee_amount)
        .ok_or(ErrorCode::Overflow.into())
}

/// Check minimum collateral ratio (equivalent to INJECTIVE's check_trove_icr_with_ratio)
pub fn check_minimum_collateral_ratio(ratio: u64, minimum: u8) -> Result<()> {
    require!(
        ratio >= minimum as u64 * 100, // Convert percentage to basis points
        crate::error::AerospacerProtocolError::CollateralBelowMinimum
    );
    Ok(())
}

/// Calculate liquidation reward (equivalent to INJECTIVE's liquidation reward calculation)
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

/// Calculate stake percentage (equivalent to INJECTIVE's stake percentage calculation)
pub fn calculate_stake_percentage(
    total_stake: u64,
    user_stake: u64,
) -> Result<u64> {
    if total_stake == 0 {
        return Ok(0);
    }
    
    user_stake
        .checked_mul(10000) // Convert to basis points
        .ok_or(ErrorCode::Overflow)?
        .checked_div(total_stake)
        .ok_or(ErrorCode::Overflow.into())
}

/// Calculate debt distribution ratio for trove liquidation
pub fn calculate_debt_distribution_ratio(
    trove_debt: u64,
    total_debt: u64,
) -> Result<u64> {
    if total_debt == 0 {
        return Ok(0);
    }
    
    trove_debt
        .checked_mul(10000) // Convert to basis points
        .ok_or(ErrorCode::Overflow)?
        .checked_div(total_debt)
        .ok_or(ErrorCode::Overflow.into())
}

/// Calculate collateral distribution amount for trove liquidation
pub fn calculate_collateral_distribution(
    total_collateral: u64,
    distribution_ratio: u64,
) -> Result<u64> {
    total_collateral
        .checked_mul(distribution_ratio)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(10000) // Convert from basis points
        .ok_or(ErrorCode::Overflow.into())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
} 