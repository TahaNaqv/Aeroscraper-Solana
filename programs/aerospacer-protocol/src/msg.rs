use anchor_lang::prelude::*;

// Initialize message (equivalent to INJECTIVE's InstantiateMsg)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeMsg {
    pub stable_coin_mint: Pubkey,
    pub oracle_program: Pubkey,
    pub fee_distributor: Pubkey,
}

// Execute messages (equivalent to INJECTIVE's ExecuteMsg)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct OpenTroveMsg {
    pub loan_amount: u64,
    pub collateral_amount: u64,
    pub collateral_denom: String,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddCollateralMsg {
    pub amount: u64,
    pub collateral_denom: String,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveCollateralMsg {
    pub amount: u64,
    pub collateral_denom: String,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BorrowLoanMsg {
    pub amount: u64,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RepayLoanMsg {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LiquidateTrovesMsg {
    pub max_troves: u32,
    pub liquidation_list: Vec<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeMsg {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UnstakeMsg {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RedeemMsg {
    pub amount: u64,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawLiquidationGainsMsg {
    pub collateral_denom: String,
    pub amount: u64,
}

// Query messages (equivalent to INJECTIVE's QueryMsg)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TroveQueryMsg {
    pub user: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StateQueryMsg {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeQueryMsg {
    pub user: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigQueryMsg {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TotalCollateralAmountsQueryMsg {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TotalDebtAmountQueryMsg {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TotalStakeAmountQueryMsg {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LiquidationGainsQueryMsg {
    pub user: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FindSortedTroveInsertPositionQueryMsg {
    pub icr: u64,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

// Response structures (equivalent to INJECTIVE's response structs)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TroveResponse {
    pub collateral_amount: u64,
    pub debt_amount: u64,
    pub collateral_ratio: u64,
    pub collateral_denom: String,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StateResponse {
    pub total_debt_amount: u64,
    pub total_stake_amount: u64,
    pub minimum_collateral_ratio: u64,
    pub protocol_fee: u64,
    pub total_collateral_amounts: Vec<u64>,
    pub collateral_denoms: Vec<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeResponse {
    pub stake_amount: u64,
    pub stake_percentage: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigResponse {
    pub admin: Pubkey,
    pub stable_coin_mint: Pubkey,
    pub oracle_program: Pubkey,
    pub fee_distributor: Pubkey,
    pub minimum_collateral_ratio: u8,
    pub protocol_fee: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CollateralAmountResponse {
    pub denom: String,
    pub amount: u64,
}

// Receive message for CW20 contract (equivalent to INJECTIVE's ReceiveMsg)
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ReceiveMsg {
    pub action: String,
    pub amount: u64,
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
} 