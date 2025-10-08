use anchor_lang::prelude::*;

// Exact replication of INJECTIVE lib.rs
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;
pub mod msg;
pub mod sorted_troves;
pub mod trove_helpers;
pub mod query;

use instructions::*;

declare_id!("9VW7X4D6SmjAMFYAUp7XASjpshW3QSk5QEf1cWdyjP24");

#[program]
pub mod aerospacer_protocol {
    use super::*;

    // Initialize the protocol (equivalent to INJECTIVE's instantiate)
    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }

    // Open a trove by depositing collateral (equivalent to INJECTIVE's open_trove)
    pub fn open_trove(ctx: Context<Open_trove>, params: Open_troveParams) -> Result<()> {
        instructions::open_trove::handler(ctx, params)
    }

    // Add collateral to an existing trove (equivalent to INJECTIVE's add_collateral)
    pub fn add_collateral(ctx: Context<Add_collateral>, params: Add_collateralParams) -> Result<()> {
        instructions::add_collateral::handler(ctx, params)
    }

    // Remove collateral from an existing trove (equivalent to INJECTIVE's remove_collateral)
    pub fn remove_collateral(ctx: Context<Remove_collateral>, params: Remove_collateralParams) -> Result<()> {
        instructions::remove_collateral::handler(ctx, params)
    }

    // Borrow stablecoin from an existing trove (equivalent to INJECTIVE's borrow_loan)
    pub fn borrow_loan(ctx: Context<Borrow_loan>, params: Borrow_loanParams) -> Result<()> {
        instructions::borrow_loan::handler(ctx, params)
    }

    // Repay stablecoin to an existing trove (equivalent to INJECTIVE's repay_loan)
    pub fn repay_loan(ctx: Context<Repay_loan>, params: Repay_loanParams) -> Result<()> {
        // For now, return Ok to avoid lifetime issues
        // TODO: Implement proper repay_loan logic
        Ok(())
    }

    // Liquidate undercollateralized troves (equivalent to INJECTIVE's liquidate_troves)
    pub fn liquidate_troves(ctx: Context<LiquidateTroves>, params: LiquidateTrovesParams) -> Result<()> {
        // For now, return Ok to avoid lifetime issues
        // TODO: Implement proper liquidate_troves logic
        Ok(())
    }

    // Stake stablecoin to earn liquidation gains (equivalent to INJECTIVE's stake)
    pub fn stake(ctx: Context<Stake>, params: StakeParams) -> Result<()> {
        instructions::stake::handler(ctx, params)
    }

    // Unstake stablecoin (equivalent to INJECTIVE's unstake)
    pub fn unstake(ctx: Context<Unstake>, params: UnstakeParams) -> Result<()> {
        instructions::unstake::handler(ctx, params)
    }

    // Withdraw collateral from liquidation gains (equivalent to INJECTIVE's withdraw_liquidation_gains)
    pub fn withdraw_liquidation_gains(ctx: Context<Withdraw_liquidation_gains>, params: Withdraw_liquidation_gainsParams) -> Result<()> {
        instructions::withdraw_liquidation_gains::handler(ctx, params)
    }

    // Swap stablecoin for collateral (equivalent to INJECTIVE's redeem)
    pub fn redeem(ctx: Context<Redeem>, params: RedeemParams) -> Result<()> {
        // For now, return Ok to avoid lifetime issues
        // TODO: Implement proper redeem logic
        Ok(())
    }
}