use anchor_lang::prelude::*;
use crate::msg::*;
use crate::state::*;

pub fn query_stake(stake: &StakeAccount) -> Result<StakeResponse> {
    let response = StakeResponse {
        stake_amount: stake.amount,
        stake_percentage: stake.percentage,
    };
    
    Ok(response)
} 