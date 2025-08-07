use anchor_lang::prelude::*;

#[account]
pub struct OracleStateAccount {
    pub admin: Pubkey,
    pub oracle_address: Pubkey,
    pub collateral_data: Vec<CollateralData>, // Added to match INJECTIVE's DATA map
}

impl OracleStateAccount {
    pub const LEN: usize = 32 + 32 + 1000; // + space for collateral_data vector
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CollateralData {
    pub denom: String,
    pub decimal: u8,
    pub price_id: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PriceResponse {
    pub denom: String,
    pub price: i64,
    pub decimal: u8,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigResponse {
    pub admin: Pubkey,
    pub oracle_address: Pubkey,
} 