use anchor_lang::prelude::*;

#[error_code]
pub enum AerospacerOracleError {
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Price feed not found")]
    PriceFeedNotFound,
    
    #[msg("Invalid price data")]
    InvalidPriceData,
    
    #[msg("Price too old")]
    PriceTooOld,
} 