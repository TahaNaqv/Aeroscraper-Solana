pub mod stake;
pub mod funds;
pub mod oracle;
pub mod fees;

pub use stake::*;
pub use funds::*;
pub use oracle::*;
pub use fees::*;

// Re-export common types and functions for easy access
pub use anchor_lang::prelude::*;

// Common constants
pub const BASIS_POINTS: u64 = 10000; // 100% = 10000 basis points
pub const MINIMUM_COLLATERAL_RATIO: u64 = 11500; // 115% in basis points
pub const DEFAULT_PROTOCOL_FEE: u64 = 500; // 5% in basis points
pub const MINIMUM_LOAN_AMOUNT: u64 = 1_000_000_000_000_000_000; // 1 aUSD with 18 decimals

// Common utility functions
pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b).ok_or(UtilsError::Overflow.into())
}

pub fn safe_div(a: u64, b: u64) -> Result<u64> {
    if b == 0 {
        return Err(UtilsError::DivisionByZero.into());
    }
    a.checked_div(b).ok_or(UtilsError::Overflow.into())
}

pub fn safe_add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or(UtilsError::Overflow.into())
}

pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b).ok_or(UtilsError::Overflow.into())
}

#[error_code]
pub enum UtilsError {
    #[msg("Overflow occurred")]
    Overflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Invalid parameter")]
    InvalidParameter,
} 