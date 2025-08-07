use anchor_lang::prelude::*;

// Initialize message
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeMsg {
    pub admin: Pubkey,
    pub stake_contract: Pubkey,
}

// Execute messages
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ToggleStakeContractMsg {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetStakeContractAddressMsg {
    pub new_address: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DistributeFeeMsg {
    pub amount: u64,
}

// Query messages
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetFeeStateMsg {}

// Response structures
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FeeStateResponse {
    pub total_fees_collected: u64,
    pub stake_contract_enabled: bool,
    pub stake_contract_address: Pubkey,
} 