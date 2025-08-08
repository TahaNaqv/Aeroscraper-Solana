use anchor_lang::prelude::*;

// Initialize message (equivalent to INJECTIVE's InstantiateMsg)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeMsg {
    pub admin: Pubkey,
    pub stake_contract: Pubkey,
}

// Execute messages (equivalent to INJECTIVE's ExecuteMsg)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ToggleStakeContractMsg {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetStakeContractAddressMsg {
    pub address: String, // Equivalent to INJECTIVE's address: String
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DistributeFeeMsg {}

// Query messages (equivalent to INJECTIVE's QueryMsg)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetConfigMsg {}

// Response structures (equivalent to INJECTIVE's ConfigResponse)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigResponse {
    pub admin: Pubkey,
    pub is_stake_enabled: bool,
    pub stake_contract_address: Pubkey,
    pub total_fees_collected: u64,
} 