use anchor_lang::prelude::*;

/// Check if funds are valid (equivalent to INJECTIVE's check_funds)
pub fn check_funds(
    token_amount: u64,
    expected_amount: u64,
    token_mint: Pubkey,
    expected_mint: Pubkey,
) -> Result<()> {
    if token_amount == 0 {
        return Err(FundsError::MissingFunds.into());
    }
    
    if token_mint != expected_mint {
        return Err(FundsError::InvalidMint.into());
    }
    
    if token_amount != expected_amount {
        return Err(FundsError::InvalidAmount.into());
    }
    
    Ok(())
}

/// Check single token transfer (equivalent to INJECTIVE's check_single_coin)
pub fn check_single_token(
    token_amount: u64,
    expected_amount: u64,
    token_mint: Pubkey,
    expected_mint: Pubkey,
) -> Result<()> {
    if token_amount == 0 {
        return Err(FundsError::MissingFunds.into());
    }
    
    if token_mint != expected_mint {
        return Err(FundsError::InvalidMint.into());
    }
    
    if token_amount != expected_amount {
        return Err(FundsError::InvalidAmount.into());
    }
    
    Ok(())
}

/// Validate collateral parameters
pub fn validate_collateral_params(
    collateral_amount: u64,
    collateral_mint: Pubkey,
    minimum_amount: u64,
) -> Result<()> {
    if collateral_amount < minimum_amount {
        return Err(FundsError::InsufficientCollateral.into());
    }
    
    if collateral_mint == Pubkey::default() {
        return Err(FundsError::InvalidMint.into());
    }
    
    Ok(())
}

/// Calculate protocol fee (equivalent to INJECTIVE's fee calculation)
pub fn calculate_protocol_fee(amount: u64, fee_rate: u64) -> Result<u64> {
    amount
        .checked_mul(fee_rate)
        .ok_or(FundsError::Overflow)?
        .checked_div(10000) // fee_rate in basis points
        .ok_or(FundsError::Overflow.into())
}

/// Calculate net amount after protocol fee
pub fn calculate_net_amount(amount: u64, fee_rate: u64) -> Result<u64> {
    let fee_amount = calculate_protocol_fee(amount, fee_rate)?;
    amount
        .checked_sub(fee_amount)
        .ok_or(FundsError::Overflow.into())
}

#[error_code]
pub enum FundsError {
    #[msg("Missing funds")]
    MissingFunds,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Invalid funds")]
    InvalidFunds,
    #[msg("Extra funds found")]
    ExtraFunds,
} 