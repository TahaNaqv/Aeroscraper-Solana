use anchor_lang::prelude::*;

/// Calculate collateral ratio (equivalent to INJECTIVE's get_trove_icr)
pub fn calculate_collateral_ratio(collateral_amount: u64, debt_amount: u64, collateral_price: u64) -> Result<u64> {
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

/// Check if trove can be liquidated (equivalent to INJECTIVE's can_liquidate_trove)
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

/// Calculate protocol fee (equivalent to INJECTIVE's fee calculation)
pub fn calculate_protocol_fee(amount: u64, fee_rate: u64) -> Result<u64> {
    amount
        .checked_mul(fee_rate)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(10000) // fee_rate in basis points
        .ok_or(ErrorCode::Overflow.into())
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
        return Err(ErrorCode::InvalidTroveParameters.into());
    }
    
    if debt_amount > 0 {
        let collateral_ratio = calculate_collateral_ratio(collateral_amount, debt_amount, collateral_price)?;
        if collateral_ratio < minimum_collateral_ratio {
            return Err(ErrorCode::InsufficientCollateralRatio.into());
        }
    }
    
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

/// Query collateral price (placeholder - will integrate with Pyth)
pub fn query_collateral_price(
    _oracle_program: Pubkey,
    _collateral_denom: String,
) -> Result<u64> {
    // TODO: Integrate with Pyth Network for real price feeds
    // For now, return placeholder price
    Ok(1_000_000) // $1.00 with 6 decimals
}

/// Query all collateral prices (equivalent to INJECTIVE's query_all_collateral_prices)
pub fn query_all_collateral_prices(
    _oracle_program: Pubkey,
) -> Result<Vec<u64>> {
    // TODO: Integrate with Pyth Network for real price feeds
    // For now, return placeholder prices
    Ok(vec![1_000_000, 2_000_000, 3_000_000]) // Sample prices
}

/// Process protocol fees (equivalent to INJECTIVE's process_protocol_fees)
pub fn process_protocol_fees(
    _fee_distributor: Pubkey,
    _fee_amount: u64,
    _collateral_denom: String,
) -> Result<()> {
    // TODO: Implement fee distribution logic
    msg!("Processing protocol fees: {} {}", _fee_amount, _collateral_denom);
    Ok(())
}

/// Populate fee coins (equivalent to INJECTIVE's populate_fee_coins)
pub fn populate_fee_coins(
    _fee_amount: u64,
    _collateral_denom: String,
) -> Result<Vec<u64>> {
    // TODO: Implement fee coin population logic
    Ok(vec![_fee_amount])
}

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Invalid trove parameters")]
    InvalidTroveParameters,
    #[msg("Insufficient collateral ratio")]
    InsufficientCollateralRatio,
} 