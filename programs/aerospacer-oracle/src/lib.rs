use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod msg;

use instructions::*;
use crate::state::PriceResponse;

declare_id!("7kJZg8PDkutRui282AnspEnLcyExxcpsbCvyfBoTcDwN");

#[program]
pub mod aerospacer_oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }

    pub fn update_oracle_address(ctx: Context<UpdateOracleAddress>, params: UpdateOracleAddressParams) -> Result<()> {
        instructions::update_oracle_address::handler(ctx, params)
    }

    pub fn set_data(ctx: Context<SetData>, params: SetDataParams) -> Result<()> {
        instructions::set_data::handler(ctx, params)
    }

    pub fn set_data_batch(ctx: Context<SetDataBatch>, params: SetDataBatchParams) -> Result<()> {
        instructions::set_data_batch::handler(ctx, params)
    }

    pub fn remove_data(ctx: Context<RemoveData>, params: RemoveDataParams) -> Result<()> {
        instructions::remove_data::handler(ctx, params)
    }

    pub fn get_price(ctx: Context<GetPrice>, params: GetPriceParams) -> Result<PriceResponse> {
        instructions::get_price::handler(ctx, params)
    }
} 