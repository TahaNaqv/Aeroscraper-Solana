use anchor_lang::prelude::*;

/// Query collateral price from oracle (placeholder for Pyth integration)
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

/// Validate price data
pub fn validate_price_data(
    price: u64,
    min_price: u64,
    max_price: u64,
) -> Result<()> {
    if price < min_price || price > max_price {
        return Err(OracleError::InvalidPrice.into());
    }
    
    if price == 0 {
        return Err(OracleError::ZeroPrice.into());
    }
    
    Ok(())
}

/// Check if price is stale (older than max_age)
pub fn is_price_stale(
    price_timestamp: i64,
    max_age: i64,
) -> Result<bool> {
    let current_time = Clock::get()?.unix_timestamp;
    let age = current_time - price_timestamp;
    
    Ok(age > max_age)
}

/// Calculate price impact for large trades
pub fn calculate_price_impact(
    trade_amount: u64,
    total_liquidity: u64,
) -> Result<u64> {
    if total_liquidity == 0 {
        return Err(OracleError::ZeroLiquidity.into());
    }
    
    let impact = trade_amount
        .checked_mul(10000) // Convert to basis points
        .ok_or(OracleError::Overflow)?
        .checked_div(total_liquidity)
        .ok_or(OracleError::Overflow)?;
    
    Ok(impact)
}

#[error_code]
pub enum OracleError {
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Zero price")]
    ZeroPrice,
    #[msg("Stale price")]
    StalePrice,
    #[msg("Zero liquidity")]
    ZeroLiquidity,
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Oracle query failed")]
    OracleQueryFailed,
} 