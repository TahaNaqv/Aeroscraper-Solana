use anchor_lang::prelude::*;

/// Main oracle state account containing all configuration and data
#[account]
pub struct OracleStateAccount {
    /// Contract administrator with privileged access
    pub admin: Pubkey,
    
    /// External oracle provider address (e.g., Pyth Network)
    pub oracle_address: Pubkey,
    
    /// Vector of supported collateral assets and their configuration
    pub collateral_data: Vec<CollateralData>,
    
    /// Timestamp of last state update
    pub last_update: i64,
}

impl OracleStateAccount {
    /// Calculate required account space
    pub const LEN: usize = 32 + 32 + 1000 + 8; // admin + oracle_address + collateral_data + last_update
}

/// Collateral asset data structure for oracle integration
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CollateralData {
    /// Asset denomination (e.g., "inj", "atom", "sol")
    pub denom: String,
    
    /// Decimal precision for price calculations (6, 18, etc.)
    pub decimal: u8,
    
    /// Pyth Network price feed identifier (hex format)
    /// Example: "0x2f95862b045670cd22bee3114c39763a34a94be1d3d9e600dfe3238c6f7bcef3"
    pub price_id: String,
    
    /// Timestamp when this asset was last configured
    pub configured_at: i64,
    
    /// Pyth price account address for this asset
    pub pyth_price_account: Pubkey,
}

/// Price response containing real-time asset price data
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PriceResponse {
    /// Asset denomination (e.g., "inj", "atom")
    pub denom: String,
    
    /// Current real-time price from oracle (scaled by decimals)
    pub price: i64,
    
    /// Decimal precision for price calculations
    pub decimal: u8,
    
    /// Timestamp when price was fetched
    pub timestamp: i64,
    
    /// Price confidence interval (from Pyth)
    pub confidence: u64,
    
    /// Price exponent (from Pyth)
    pub exponent: i32,
}

/// Configuration response containing contract settings
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigResponse {
    /// Contract administrator address
    pub admin: Pubkey,
    
    /// Current external oracle provider address
    pub oracle_address: Pubkey,
    
    /// Number of supported collateral assets
    pub asset_count: u32,
    
    /// Timestamp of last configuration update
    pub last_update: i64,
}