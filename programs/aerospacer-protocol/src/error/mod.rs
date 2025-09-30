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
    
    #[msg("Invalid reply ID")]
    InvalidReplyID,
    
    #[msg("Error while instantiating cw20 contract")]
    ReplyError,
    
    #[msg("Invalid collateral denom")]
    InvalidCollateralDenom,
    
    #[msg("Trove not active")]
    TroveNotActive,
    
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
    
    #[msg("Invalid redemption amount")]
    InvalidRedemptionAmount,
    
    #[msg("Redemption failed")]
    RedemptionFailed,
    
    #[msg("Invalid liquidation parameters")]
    InvalidLiquidationParameters,
    
    #[msg("No troves to liquidate")]
    NoTrovesToLiquidate,
    
    #[msg("Liquidation failed")]
    LiquidationFailed,
    
    #[msg("Invalid liquidation list")]
    InvalidLiquidationList,
    
    #[msg("Stake amount too small")]
    StakeAmountTooSmall,
    
    #[msg("Invalid stake parameters")]
    InvalidStakeParameters,
    
    #[msg("Insufficient stake")]
    InsufficientStake,
    
    #[msg("Invalid unstake amount")]
    InvalidUnstakeAmount,
    
    #[msg("No liquidation gains found")]
    NoLiquidationGains,
    
    #[msg("Gains already claimed")]
    GainsAlreadyClaimed,
    
    #[msg("Withdrawal failed")]
    WithdrawalFailed,
    
    #[msg("Insufficient gains")]
    InsufficientGains,
    
    #[msg("Overflow occurred")]
    Overflow,
    
    #[msg("Invalid trove parameters")]
    InvalidTroveParameters,
    
    #[msg("Insufficient collateral ratio")]
    InsufficientCollateralRatio,
    
    #[msg("Account not found")]
    AccountNotFound,
    
    #[msg("Trove not liquidatable")]
    TroveNotLiquidatable,
} 