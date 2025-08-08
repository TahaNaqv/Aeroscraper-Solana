use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;
pub mod msg;
pub mod sorted_troves;
pub mod trove_helpers;
pub mod query;

use instructions::*;

declare_id!("mR3CUXYeYLjoxFJ1ieBfC9rLciZwe8feFYvXKdafihD");

#[program]
pub mod aerospacer_protocol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }

    pub fn open_trove(ctx: Context<Open_trove>, params: Open_troveParams) -> Result<()> {
        instructions::open_trove::handler(ctx, params)
    }

    pub fn add_collateral(ctx: Context<Add_collateral>, params: Add_collateralParams) -> Result<()> {
        instructions::add_collateral::handler(ctx, params)
    }

    pub fn remove_collateral(ctx: Context<Remove_collateral>, params: Remove_collateralParams) -> Result<()> {
        instructions::remove_collateral::handler(ctx, params)
    }

    pub fn borrow_loan(ctx: Context<Borrow_loan>, params: Borrow_loanParams) -> Result<()> {
        instructions::borrow_loan::handler(ctx, params)
    }

    pub fn repay_loan(ctx: Context<Repay_loan>, params: Repay_loanParams) -> Result<()> {
        instructions::repay_loan::handler(ctx, params)
    }

    pub fn liquidate_troves(ctx: Context<Liquidate_troves>, params: Liquidate_trovesParams) -> Result<()> {
        instructions::liquidate_troves::handler(ctx, params)
    }

    pub fn stake(ctx: Context<Stake>, params: StakeParams) -> Result<()> {
        instructions::stake::handler(ctx, params)
    }

    pub fn unstake(ctx: Context<Unstake>, params: UnstakeParams) -> Result<()> {
        instructions::unstake::handler(ctx, params)
    }

    pub fn withdraw_liquidation_gains(ctx: Context<Withdraw_liquidation_gains>, params: Withdraw_liquidation_gainsParams) -> Result<()> {
        instructions::withdraw_liquidation_gains::handler(ctx, params)
    }

    pub fn redeem(ctx: Context<Redeem>, params: RedeemParams) -> Result<()> {
        instructions::redeem::handler(ctx, params)
    }
} 