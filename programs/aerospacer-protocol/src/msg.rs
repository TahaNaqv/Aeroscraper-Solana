use anchor_lang::prelude::*;

// Initialize message
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeMsg {
    pub stable_coin_mint: Pubkey,
    pub oracle_program: Pubkey,
    pub fee_distributor: Pubkey,
}

// Execute messages
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct OpenTroveMsg {
    pub collateral_amount: u64,
    pub debt_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddCollateralMsg {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveCollateralMsg {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BorrowLoanMsg {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RepayLoanMsg {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LiquidateTrovesMsg {
    pub max_troves: u32,
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
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawLiquidationGainsMsg {
    pub amount: u64,
}

// Query messages
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

// Response structures
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TroveResponse {
    pub collateral_amount: u64,
    pub debt_amount: u64,
    pub collateral_ratio: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StateResponse {
    pub total_debt_amount: u64,
    pub total_stake_amount: u64,
    pub minimum_collateral_ratio: u64,
    pub protocol_fee: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeResponse {
    pub stake_amount: u64,
    pub stake_percentage: u64,
} 