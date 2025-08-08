use anchor_lang::prelude::*;

// Hardcoded fee addresses for Solana (following INJECTIVE project pattern)
// FEE_ADDR_1: Protocol Treasury/Development Fund
// FEE_ADDR_2: Validator Rewards/Staking Pool
pub const FEE_ADDR_1: &str = "8Lv4UrYHTrzvg9jPVVGNmxWyMrMvrZnCQLWucBzfJyyR";
pub const FEE_ADDR_2: &str = "GcNwV1nA5bityjNYsWwPLHykpKuuhPzK1AQFBbrPopnX";

#[account]
pub struct FeeStateAccount {
    pub admin: Pubkey,
    pub is_stake_enabled: bool,
    pub stake_contract_address: Pubkey,
    pub total_fees_collected: u64,
}

impl FeeStateAccount {
    pub const LEN: usize = 32 + 1 + 32 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigResponse {
    pub admin: Pubkey,
    pub is_stake_enabled: bool,
    pub stake_contract_address: Pubkey,
    pub total_fees_collected: u64,
} 