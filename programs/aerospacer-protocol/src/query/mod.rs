pub mod trove;
pub mod state;
pub mod stake;

pub use trove::*;
pub use state::*;
pub use stake::*;

use anchor_lang::prelude::*;
use crate::state::*;
use crate::msg::*;

/// Query total collateral amounts (equivalent to INJECTIVE's query_total_collateral_amounts)
pub fn query_total_collateral_amounts<'a>(
    state: &StateAccount,
    program_id: &Pubkey,
    remaining_accounts: &'a [AccountInfo<'a>],
) -> Result<Vec<CollateralAmountResponse>> {
    let mut responses = Vec::new();
    
    for denom in &state.collateral_denoms {
        let amount = crate::utils::get_total_collateral_amount(denom, program_id, remaining_accounts)?;
        responses.push(CollateralAmountResponse {
            denom: denom.clone(),
            amount,
        });
    }
    
    Ok(responses)
}

/// Query total debt amount (equivalent to INJECTIVE's query_total_debt_amount)
pub fn query_total_debt_amount(state: &StateAccount) -> Result<u64> {
    Ok(state.total_debt_amount)
}

/// Query total stake amount (equivalent to INJECTIVE's query_total_stake_amount)
pub fn query_total_stake_amount(state: &StateAccount) -> Result<u64> {
    Ok(state.total_stake_amount)
}

/// Query liquidation gains (equivalent to INJECTIVE's query_liquidation_gains)
pub fn query_liquidation_gains(
    _user: Pubkey,
    _state: &StateAccount,
) -> Result<u64> {
    // TODO: Implement liquidation gains calculation
    // In real implementation, you'd calculate based on user's stake and liquidation events
    Ok(0) // Placeholder
}

/// Find sorted trove insert position (equivalent to INJECTIVE's query_find_sorted_troves_insert_position)
pub fn query_find_sorted_troves_insert_position(
    _icr: u64,
    _prev_node_id: Option<Pubkey>,
    _next_node_id: Option<Pubkey>,
) -> Result<(Option<Pubkey>, Option<Pubkey>)> {
    // TODO: Implement sorted troves position finding
    // In real implementation, you'd traverse the sorted list to find proper position
    Ok((_prev_node_id, _next_node_id)) // Placeholder
} 