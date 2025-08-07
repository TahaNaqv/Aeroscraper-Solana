use anchor_lang::prelude::*;

// Initialize message
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeMsg {
    pub admin: Pubkey,
}

// Execute messages
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateOracleAddressMsg {
    pub new_address: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetDataMsg {
    pub price: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetDataBatchMsg {
    pub prices: Vec<u64>,
    pub timestamps: Vec<i64>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveDataMsg {
    pub index: u32,
}

// Query messages
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetPriceMsg {
    pub symbol: String,
}

// Response structures
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PriceResponse {
    pub price: u64,
    pub timestamp: i64,
    pub symbol: String,
} 