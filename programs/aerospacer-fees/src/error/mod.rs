use anchor_lang::prelude::*;

#[error_code]
pub enum AerospacerFeesError {
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Transfer failed")]
    TransferFailed,
} 