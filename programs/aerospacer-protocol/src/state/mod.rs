use anchor_lang::prelude::*;

#[account]
pub struct StateAccount {
    pub admin: Pubkey,
    pub stable_coin_mint: Pubkey,
    pub oracle_program: Pubkey,
    pub fee_distributor: Pubkey,
    pub minimum_collateral_ratio: u8,
    pub protocol_fee: u8,
    pub total_debt_amount: u64,
    pub total_stake_amount: u64,
    // Additional fields to match INJECTIVE
    pub collateral_denoms: Vec<String>,     // List of supported collateral types
    // Note: total_collateral_amounts moved to per-denom PDA accounts
}

impl StateAccount {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 1 + 1 + 8 + 8 + 32; // + space for collateral_denoms vector
}

#[account]
pub struct TroveAccount {
    pub owner: Pubkey,
    pub debt_amount: u64,
    pub collateral_amount: u64,
    pub collateral_ratio: u64,
    pub created_at: i64,
    // Additional fields to match INJECTIVE
    pub collateral_denom: String,
    pub is_active: bool,
}

impl TroveAccount {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 8 + 32 + 1; // + space for string
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount: u64,
    pub percentage: u64,
    // Additional fields to match INJECTIVE
    pub total_stake_at_time: u64,
    pub block_height: u64,
}

impl StakeAccount {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 8;
}

#[account]
pub struct LiquidationGainsAccount {
    pub user: Pubkey,
    pub block_height: u64,
    pub collateral_gain: u64,
    pub collateral_denom: String,
    pub is_claimed: bool,
}

impl LiquidationGainsAccount {
    pub const LEN: usize = 32 + 8 + 8 + 32 + 1; // + space for string
}

// Per-denom collateral totals PDA account
#[account]
pub struct TotalCollateralByDenom {
    pub denom: String,
    pub total_amount: u64,
    pub last_updated: i64,
}

impl TotalCollateralByDenom {
    pub const LEN: usize = 32 + 8 + 8; // + space for string
    
    pub fn pda_seeds(denom: &str) -> [&[u8]; 2] {
        [b"total_collateral", denom.as_bytes()]
    }
}

// Protocol vault PDA accounts for secure token operations
#[account]
pub struct ProtocolVault {
    pub vault_type: u8, // 0 = stablecoin, 1 = collateral
    pub denom: String,  // For collateral vaults
    pub total_deposited: u64,
    pub total_withdrawn: u64,
}

impl ProtocolVault {
    pub const LEN: usize = 1 + 32 + 8 + 8; // + space for string
    
    pub fn stablecoin_vault_seeds() -> [&'static [u8]; 1] {
        [b"protocol_stablecoin_vault"]
    }
    
    pub fn collateral_vault_seeds(denom: &str) -> [&[u8]; 2] {
        [b"protocol_collateral_vault", denom.as_bytes()]
    }
}

// Constants to match INJECTIVE
pub const MINIMUM_LOAN_AMOUNT: u64 = 1_000_000_000_000_000_000; // 1 aUSD with 18 decimals
pub const MINIMUM_COLLATERAL_AMOUNT: u64 = 5_000_000_000; // 5 SOL with 9 decimals
pub const DEFAULT_MINIMUM_COLLATERAL_RATIO: u8 = 115; // 115%
pub const DEFAULT_PROTOCOL_FEE: u8 = 5; // 5% 