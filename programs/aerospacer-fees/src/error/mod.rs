use anchor_lang::prelude::*;

#[error_code]
pub enum AerospacerFeesError {
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("No fees to distribute")]
    NoFeesToDistribute,
    
    #[msg("Overflow occurred")]
    Overflow,
    
    #[msg("Invalid fee distribution")]
    InvalidFeeDistribution,
    
    #[msg("Transfer failed")]
    TransferFailed,
} 