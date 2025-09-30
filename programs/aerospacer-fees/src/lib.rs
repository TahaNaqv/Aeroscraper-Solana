use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod msg;

use instructions::*;
use crate::state::ConfigResponse;
use crate::instructions::distribute_fee::DistributeFeeParams;

declare_id!("3nbhQ7bahEr733uiBYKmTgnuGFzCCnc6JDkpZDjXdomC");

#[program]
pub mod aerospacer_fees {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn toggle_stake_contract(ctx: Context<ToggleStakeContract>) -> Result<()> {
        instructions::toggle_stake_contract::handler(ctx)
    }

    pub fn set_stake_contract_address(ctx: Context<SetStakeContractAddress>, params: SetStakeContractAddressParams) -> Result<()> {
        instructions::set_stake_contract_address::handler(ctx, params)
    }

    pub fn distribute_fee(ctx: Context<DistributeFee>, params: DistributeFeeParams) -> Result<()> {
        instructions::distribute_fee::handler(ctx, params)
    }

    pub fn get_config(ctx: Context<GetConfig>) -> Result<ConfigResponse> {
        instructions::get_config::handler(ctx)
    }
} 