use anchor_lang::prelude::*;

pub fn calculate_collateral_ratio(
    collateral_value: u64,
    debt_amount: u64,
) -> Result<u64> {
    if debt_amount == 0 {
        return Ok(0);
    }
    Ok((collateral_value * 100) / debt_amount)
}

pub fn calculate_protocol_fee(amount: u64, fee_rate: u8) -> Result<u64> {
    Ok((amount * fee_rate as u64) / 1000)
}

pub fn check_minimum_collateral_ratio(ratio: u64, minimum: u8) -> Result<()> {
    require!(
        ratio >= minimum as u64,
        crate::error::AerospacerProtocolError::CollateralBelowMinimum
    );
    Ok(())
} 