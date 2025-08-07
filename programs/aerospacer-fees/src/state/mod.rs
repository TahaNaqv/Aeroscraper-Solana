use anchor_lang::prelude::*;

#[account]
pub struct FeeStateAccount {
    pub admin: Pubkey,
    pub is_stake_enabled: bool,
    pub stake_contract_address: Pubkey,
    pub total_fees_collected: u64, // Added to track total fees collected
    pub fee_address_1: Pubkey,     // Added for fee distribution
    pub fee_address_2: Pubkey,     // Added for fee distribution
}

impl FeeStateAccount {
    pub const LEN: usize = 32 + 1 + 32 + 8 + 32 + 32; // Updated size
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigResponse {
    pub admin: Pubkey,
    pub is_stake_enabled: bool,
    pub stake_contract_address: Pubkey,
    pub total_fees_collected: u64,
    pub fee_address_1: Pubkey,
    pub fee_address_2: Pubkey,
} 