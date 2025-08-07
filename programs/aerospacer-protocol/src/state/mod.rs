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
    pub total_collateral_amounts: Vec<u64>, // Map of denom -> amount
    pub collateral_denoms: Vec<String>,     // List of supported collateral types
}

impl StateAccount {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 1 + 1 + 8 + 8 + 32 + 32; // + space for vectors
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

// Constants to match INJECTIVE
pub const MINIMUM_LOAN_AMOUNT: u64 = 1_000_000_000_000_000_000; // 1 aUSD with 18 decimals
pub const DEFAULT_MINIMUM_COLLATERAL_RATIO: u8 = 115; // 115%
pub const DEFAULT_PROTOCOL_FEE: u8 = 5; // 5% 