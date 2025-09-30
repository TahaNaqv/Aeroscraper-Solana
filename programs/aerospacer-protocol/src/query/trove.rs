use anchor_lang::prelude::*;
use crate::msg::*;
use crate::state::*;

pub fn query_trove(trove: &TroveAccount) -> Result<TroveResponse> {
    let response = TroveResponse {
        collateral_amount: trove.collateral_amount,
        debt_amount: trove.debt_amount,
        collateral_ratio: trove.collateral_ratio,
        collateral_denom: trove.collateral_denom.clone(),
        is_active: trove.is_active,
    };
    
    Ok(response)
} 