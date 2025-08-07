use anchor_lang::prelude::*;

/// Calculate stake amount (equivalent to INJECTIVE's calculate_stake_amount)
pub fn calculate_stake_amount(
    total_amount: u64,
    percentage: u64, // percentage in basis points (100% = 10000)
    up: bool,
) -> Result<u64> {
    let stake_amount = if up {
        // Round up calculation
        let numerator = total_amount
            .checked_mul(percentage)
            .ok_or(StakeError::Overflow)?
            .checked_add(9999) // Add 9999 to round up
            .ok_or(StakeError::Overflow)?;
        
        numerator
            .checked_div(10000)
            .ok_or(StakeError::Overflow)?
    } else {
        // Round down calculation
        total_amount
            .checked_mul(percentage)
            .ok_or(StakeError::Overflow)?
            .checked_div(10000)
            .ok_or(StakeError::Overflow)?
    };

    Ok(stake_amount)
}

/// Calculate stake percentage (equivalent to INJECTIVE's calculate_stake_percentage)
pub fn calculate_stake_percentage(
    total_amount: u64,
    stake_amount: u64,
) -> Result<u64> {
    if total_amount == 0 {
        return Err(StakeError::DivisionByZero.into());
    }

    let percentage = stake_amount
        .checked_mul(10000) // Convert to basis points
        .ok_or(StakeError::Overflow)?
        .checked_div(total_amount)
        .ok_or(StakeError::Overflow)?;

    Ok(percentage)
}

/// Calculate liquidation reward for stability pool stakers
pub fn calculate_liquidation_reward(
    debt_amount: u64,
    collateral_amount: u64,
    collateral_price: u64,
) -> Result<u64> {
    let debt_value = debt_amount; // Assuming 1:1 for stablecoin
    let collateral_value = collateral_amount
        .checked_mul(collateral_price)
        .ok_or(StakeError::Overflow)?;

    // Liquidation reward is the excess collateral
    if collateral_value > debt_value {
        collateral_value
            .checked_sub(debt_value)
            .ok_or(StakeError::Overflow.into())
    } else {
        Ok(0)
    }
}

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
        .ok_or(StakeError::Overflow)?;

    let ratio = collateral_value
        .checked_mul(10000) // 100% = 10000 basis points
        .ok_or(StakeError::Overflow)?
        .checked_div(debt_amount)
        .ok_or(StakeError::Overflow)?;

    Ok(ratio)
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

/// Validate trove parameters (equivalent to INJECTIVE's check_trove_icr_with_ratio)
pub fn validate_trove_parameters(
    collateral_amount: u64,
    debt_amount: u64,
    minimum_collateral_ratio: u64,
    collateral_price: u64,
) -> Result<()> {
    if collateral_amount == 0 && debt_amount == 0 {
        return Ok(());
    }

    if debt_amount > 0 && collateral_amount == 0 {
        return Err(StakeError::InvalidTroveParameters.into());
    }

    if debt_amount > 0 {
        let collateral_ratio = calculate_collateral_ratio(collateral_amount, debt_amount, collateral_price)?;
        if collateral_ratio < minimum_collateral_ratio {
            return Err(StakeError::InsufficientCollateralRatio.into());
        }
    }

    Ok(())
}

#[error_code]
pub enum StakeError {
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Invalid trove parameters")]
    InvalidTroveParameters,
    #[msg("Insufficient collateral ratio")]
    InsufficientCollateralRatio,
    #[msg("Invalid stake calculation")]
    InvalidStakeCalculation,
} 