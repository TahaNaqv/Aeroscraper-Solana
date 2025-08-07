use anchor_lang::prelude::*;

pub fn query_collateral_price(
    _oracle_program: Pubkey,
    _price_feed_id: Pubkey,
) -> Result<u64> {
    // Placeholder implementation
    Ok(1000000) // 1.0 with 6 decimals
}

pub fn query_all_collateral_prices(
    _oracle_program: Pubkey,
) -> Result<Vec<u64>> {
    // Placeholder implementation
    Ok(vec![1000000, 2000000, 3000000]) // Sample prices
} 