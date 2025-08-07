use anchor_lang::prelude::*;
use crate::msg::*;
use crate::state::*;

pub fn query_state(state: &StateAccount) -> Result<StateResponse> {
    let response = StateResponse {
        total_debt_amount: state.total_debt_amount,
        total_stake_amount: state.total_stake_amount,
        minimum_collateral_ratio: state.minimum_collateral_ratio as u64,
        protocol_fee: state.protocol_fee as u64,
    };
    
    Ok(response)
} 