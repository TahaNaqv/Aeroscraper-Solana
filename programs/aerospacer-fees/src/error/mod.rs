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
    
    #[msg("Invalid address")]
    InvalidAddress,
    
    #[msg("Invalid token mint - all token accounts must use the same SPL token")]
    InvalidTokenMint,
    
    #[msg("Stake contract address not set")]
    StakeContractNotSet,
    
    #[msg("Invalid stability pool account - owner must match stake_contract_address")]
    InvalidStabilityPoolAccount,
    
    #[msg("Invalid fee address 1 - owner must match FEE_ADDR_1")]
    InvalidFeeAddress1,
    
    #[msg("Invalid fee address 2 - owner must match FEE_ADDR_2")]
    InvalidFeeAddress2,
    
    #[msg("Unauthorized token account - payer must own the payer_token_account")]
    UnauthorizedTokenAccount,
} 