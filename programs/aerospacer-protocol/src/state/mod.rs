use anchor_lang::prelude::*;

// Exact replication of INJECTIVE state.rs
// Main state account (equivalent to INJECTIVE's ADMIN, ORACLE_HELPER_ADDR, FEE_DISTRIBUTOR_ADDR, MINIMUM_COLLATERAL_RATIO, PROTOCOL_FEE, STABLE_COIN_ADDR, TOTAL_DEBT_AMOUNT, TOTAL_STAKE_AMOUNT)
#[account]
pub struct StateAccount {
    pub admin: Pubkey,
    pub oracle_helper_addr: Pubkey,
    pub fee_distributor_addr: Pubkey,
    pub minimum_collateral_ratio: u8,
    pub protocol_fee: u8,
    pub stable_coin_addr: Pubkey,
    pub total_debt_amount: u64, // Equivalent to Uint256
    pub total_stake_amount: u64, // Equivalent to Uint256
}

impl StateAccount {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 1 + 1 + 32 + 8 + 8;
}

// User debt amount (equivalent to INJECTIVE's USER_DEBT_AMOUNT: Map<Addr, Uint256>)
#[account]
pub struct UserDebtAmount {
    pub owner: Pubkey,
    pub amount: u64, // Equivalent to Uint256
}

impl UserDebtAmount {
    pub const LEN: usize = 8 + 32 + 8;
    pub fn seeds(owner: &Pubkey) -> [&[u8]; 2] {
        [b"user_debt_amount", owner.as_ref()]
    }
}

// User collateral amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT: Map<(Addr, String), Uint256>)
#[account]
pub struct UserCollateralAmount {
    pub owner: Pubkey,
    pub denom: String,
    pub amount: u64, // Equivalent to Uint256
}

impl UserCollateralAmount {
    pub const LEN: usize = 8 + 32 + 32 + 8; // String length needs to be considered
    pub fn seeds<'a>(owner: &'a Pubkey, denom: &'a str) -> [&'a [u8]; 3] {
        [b"user_collateral_amount", owner.as_ref(), denom.as_bytes()]
    }
}

// User stake amount (equivalent to INJECTIVE's USER_STAKE_AMOUNT: SnapshotMap<Addr, Uint256>)
#[account]
pub struct UserStakeAmount {
    pub owner: Pubkey,
    pub amount: u64, // Equivalent to Uint256
    pub block_height: u64, // For snapshotting
}

impl UserStakeAmount {
    pub const LEN: usize = 8 + 32 + 8 + 8;
    pub fn seeds(owner: &Pubkey) -> [&[u8]; 2] {
        [b"user_stake_amount", owner.as_ref()]
    }
}

// Liquidity threshold (equivalent to INJECTIVE's LIQUIDITY_THRESHOLD: Map<Addr, Decimal256>)
#[account]
pub struct LiquidityThreshold {
    pub owner: Pubkey,
    pub ratio: u64, // Equivalent to Decimal256
}

impl LiquidityThreshold {
    pub const LEN: usize = 8 + 32 + 8;
    pub fn seeds(owner: &Pubkey) -> [&[u8]; 2] {
        [b"liquidity_threshold", owner.as_ref()]
    }
}

// Total collateral amount (equivalent to INJECTIVE's TOTAL_COLLATERAL_AMOUNT: Map<String, Uint256>)
#[account]
pub struct TotalCollateralAmount {
    pub denom: String,
    pub amount: u64, // Equivalent to Uint256
}

impl TotalCollateralAmount {
    pub const LEN: usize = 8 + 32 + 8; // String length needs to be considered
    pub fn seeds(denom: &str) -> [&[u8]; 2] {
        [b"total_collateral_amount", denom.as_bytes()]
    }
}

// User liquidation collateral gain (equivalent to INJECTIVE's USER_LIQUIDATION_COLLATERAL_GAIN: Map<(Addr, u64), bool>)
#[account]
pub struct UserLiquidationCollateralGain {
    pub user: Pubkey,
    pub block_height: u64,
    pub claimed: bool,
}

impl UserLiquidationCollateralGain {
    pub const LEN: usize = 8 + 32 + 8 + 1;
    pub fn seeds(user: &Pubkey, block_height: u64) -> [&[u8]; 3] {
        let block_height_bytes = Box::leak(block_height.to_le_bytes().to_vec().into_boxed_slice());
        [b"user_liq_gain", user.as_ref(), block_height_bytes]
    }
}

// Total liquidation collateral gain (equivalent to INJECTIVE's TOTAL_LIQUIDATION_COLLATERAL_GAIN: Map<(u64, String), Uint256>)
#[account]
pub struct TotalLiquidationCollateralGain {
    pub block_height: u64,
    pub denom: String,
    pub amount: u64, // Equivalent to Uint256
}

impl TotalLiquidationCollateralGain {
    pub const LEN: usize = 8 + 8 + 32 + 8; // String length needs to be considered
    pub fn seeds(block_height: u64, denom: &str) -> [&[u8]; 3] {
        let block_height_bytes = Box::leak(block_height.to_le_bytes().to_vec().into_boxed_slice());
        [b"total_liq_gain", block_height_bytes, denom.as_bytes()]
    }
}

// Node structure for sorted troves linked list (equivalent to INJECTIVE's NODES: Map<Addr, Node>)
#[account]
pub struct Node {
    pub id: Pubkey, // The trove owner's address
    pub prev_id: Option<Pubkey>,
    pub next_id: Option<Pubkey>,
}

impl Node {
    pub const LEN: usize = 8 + 32 + (1 + 32) + (1 + 32); // Option<Pubkey> = 1 + 32
    pub fn seeds(id: &Pubkey) -> [&[u8]; 2] {
        [b"node", id.as_ref()]
    }
}

// Sorted troves state (equivalent to INJECTIVE's HEAD, TAIL, SIZE items)
#[account]
pub struct SortedTrovesState {
    pub head: Option<Pubkey>,
    pub tail: Option<Pubkey>,
    pub size: u64,
}

impl SortedTrovesState {
    pub const LEN: usize = 8 + (1 + 32) + (1 + 32) + 8;
    pub fn seeds() -> [&'static [u8]; 1] {
        [b"sorted_troves_state"]
    }
}

// Constants to match INJECTIVE exactly
pub const MINIMUM_LOAN_AMOUNT: u64 = 1_000_000_000_000_000_000; // 1 aUSD with 18 decimals
pub const MINIMUM_COLLATERAL_AMOUNT: u64 = 5_000_000_000; // 5 SOL with 9 decimals
pub const DEFAULT_MINIMUM_COLLATERAL_RATIO: u8 = 115; // 115%
pub const DEFAULT_PROTOCOL_FEE: u8 = 5; // 5%

// Decimal fractions to match INJECTIVE
pub const DECIMAL_FRACTION_6: u128 = 1_000_000;
pub const DECIMAL_FRACTION_18: u128 = 1_000_000_000_000_000_000;