use anchor_lang::prelude::*;
use crate::msg::*;
use crate::state::*;
use crate::utils::*;

pub fn query_state<'a>(
    state: &StateAccount,
    program_id: &Pubkey,
    remaining_accounts: &'a [AccountInfo<'a>],
) -> Result<StateResponse> {
    // Collect total collateral amounts from per-denom PDAs
    let mut total_collateral_amounts = Vec::new();
    
    for denom in &state.collateral_denoms {
        let amount = get_total_collateral_amount(denom, program_id, remaining_accounts)?;
        total_collateral_amounts.push(amount);
    }
    
    let response = StateResponse {
        total_debt_amount: state.total_debt_amount,
        total_stake_amount: state.total_stake_amount,
        minimum_collateral_ratio: state.minimum_collateral_ratio as u64,
        protocol_fee: state.protocol_fee as u64,
        total_collateral_amounts,
        collateral_denoms: state.collateral_denoms.clone(),
    };
    
    Ok(response)
} 