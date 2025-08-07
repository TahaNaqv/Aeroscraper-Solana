use anchor_lang::prelude::*;

#[error_code]
pub enum AerospacerProtocolError {
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Trove already exists")]
    TroveExists,
    
    #[msg("Trove does not exist")]
    TroveDoesNotExist,
    
    #[msg("Invalid collateral ratio")]
    InvalidCollateralRatio,
    
    #[msg("Collateral below minimum")]
    CollateralBelowMinimum,
    
    #[msg("Loan amount below minimum")]
    LoanAmountBelowMinimum,
    
    #[msg("Invalid decimal")]
    InvalidDecimal,
    
    #[msg("Invalid list")]
    InvalidList,
    
    #[msg("Not enough liquidity for redeem")]
    NotEnoughLiquidityForRedeem,
    
    #[msg("No liquidation collateral rewards found")]
    CollateralRewardsNotFound,
    
    #[msg("Invalid funds")]
    InvalidFunds,
    
    #[msg("Missing funds")]
    MissingFunds,
    
    #[msg("Extra funds")]
    ExtraFunds,
} 