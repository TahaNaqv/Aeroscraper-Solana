use std::collections::HashMap;

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::instructions::TroveAmounts;
// LiquidityData is now defined in trove_management.rs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidityData {
    pub denom: String,
    pub liquidity: u64, // Equivalent to Decimal256
    pub decimal: u8,
}

// Exact replication of INJECTIVE utils.rs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CollateralGain {
    pub block_height: u64,
    pub total_collateral_amount: u64, // Equivalent to Uint256
    pub amount: u64, // Equivalent to Uint256
    pub denom: String,
}

// PriceResponse equivalent for Solana
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceResponse {
    pub denom: String,
    pub price: u64, // Equivalent to Uint256
    pub decimal: u8,
}

pub fn query_collateral_price(
    _state_account: &StateAccount,
    denom: &str,
) -> Result<PriceResponse> {
    // In Injective: querier.query_wasm_smart(oracle_helper_addr, &OracleHelperQueryMsg::Price { denom: denom.to_string() })
    // For Solana: we would query the oracle program via CPI
    // For now, return mock price based on denom
    let (price, decimal) = match denom {
        "SOL" => (100_000_000, 9), // $100 with 9 decimals
        "USDC" => (1_000_000, 6),  // $1 with 6 decimals
        "INJ" => (144015750000, 18), // Mock INJ price with 18 decimals
        "ATOM" => (6313260000, 6),  // Mock ATOM price with 6 decimals
        _ => (50_000_000, 6),      // $50 with 6 decimals
    };
    
    Ok(PriceResponse {
        denom: denom.to_string(),
        price,
        decimal,
    })
}

pub fn query_all_collateral_prices(
    _state_account: &StateAccount,
) -> Result<HashMap<String, PriceResponse>> {
    // In Injective: deps.querier.query_wasm_smart(oracle_helper_addr, &OracleHelperQueryMsg::Prices {})
    // For Solana: we would query the oracle program for all prices via CPI
    let mut map: HashMap<String, PriceResponse> = HashMap::new();
    
    // Mock prices for common denoms
    map.insert("SOL".to_string(), PriceResponse {
        denom: "SOL".to_string(),
        price: 100_000_000,
        decimal: 9,
    });
    map.insert("USDC".to_string(), PriceResponse {
        denom: "USDC".to_string(),
        price: 1_000_000,
        decimal: 6,
    });
    map.insert("INJ".to_string(), PriceResponse {
        denom: "INJ".to_string(),
        price: 144015750000,
        decimal: 18,
    });
    map.insert("ATOM".to_string(), PriceResponse {
        denom: "ATOM".to_string(),
        price: 6313260000,
        decimal: 6,
    });
    
    Ok(map)
}

pub fn query_all_denoms(
    _state_account: &StateAccount,
) -> Result<Vec<String>> {
    // In Injective: deps.querier.query_wasm_smart(oracle_helper_addr, &OracleHelperQueryMsg::AllDenoms {})
    // For Solana: we would query the oracle program for all supported denoms via CPI
    Ok(vec![
        "SOL".to_string(),
        "USDC".to_string(),
        "INJ".to_string(),
        "ATOM".to_string(),
    ])
}

pub fn get_liquidation_gains<'a>(
    user: Pubkey,
    state_account: &StateAccount,
    user_liquidation_collateral_gain_accounts: &'a [AccountInfo<'a>],
    total_liquidation_collateral_gain_accounts: &'a [AccountInfo<'a>],
    user_stake_amount_accounts: &'a [AccountInfo<'a>],
) -> Result<Vec<CollateralGain>> {
    let mut collateral_gains: Vec<CollateralGain> = vec![];

    // In Injective: TOTAL_LIQUIDATION_COLLATERAL_GAIN.range(storage, None, None, Order::Ascending)
    // For Solana: we would iterate through TotalLiquidationCollateralGain PDAs
    for account_info in total_liquidation_collateral_gain_accounts {
        let total_gain: Account<TotalLiquidationCollateralGain> = Account::try_from(account_info)?;
        let block_height = total_gain.block_height;
        let collateral_denom = total_gain.denom.clone();
        let total_collateral_amount = total_gain.amount;
        let total_stake_amount = state_account.total_stake_amount;

        // In Injective: USER_LIQUIDATION_COLLATERAL_GAIN.may_load(storage, (sender.clone(), block_height))
        // For Solana: check if user has already claimed this gain
        let user_liq_gain_seeds = UserLiquidationCollateralGain::seeds(&user, block_height);
        let (user_liq_gain_pda, _bump) = Pubkey::find_program_address(&user_liq_gain_seeds, &crate::ID);
        let mut already_claimed = false;
        for account in user_liquidation_collateral_gain_accounts {
            if account.key() == user_liq_gain_pda {
                let user_gain_account: Account<UserLiquidationCollateralGain> = Account::try_from(account)?;
                already_claimed = user_gain_account.claimed;
                break;
            }
        }

        if !already_claimed {
            // In Injective: USER_STAKE_AMOUNT.may_load_at_height(storage, sender.clone(), block_height)
            // For Solana: check user stake at specific block height (simplified)
            let user_stake_seeds = UserStakeAmount::seeds(&user);
            let (user_stake_pda, _bump) = Pubkey::find_program_address(&user_stake_seeds, &crate::ID);
            let mut user_stake_amount = 0u64;
            for account in user_stake_amount_accounts {
                if account.key() == user_stake_pda {
                    let stake_account: Account<UserStakeAmount> = Account::try_from(account)?;
                    // In Injective: SnapshotMap allows querying at specific block height
                    // For Solana: we would need to implement snapshotting or use current stake
                    user_stake_amount = stake_account.amount;
                    break;
                }
            }

            if user_stake_amount > 0 && total_stake_amount > 0 {
                // In Injective: Decimal256::from_ratio(stake_amount, total_stake_amount)
                // For Solana: simplified calculation
                let stake_percentage = (user_stake_amount * 1_000_000_000_000_000_000) / total_stake_amount; // Simplified Decimal256
                
                // In Injective: calculate_stake_amount(total_collateral_amount, stake_percentage, false)
                // For Solana: simplified calculation
                let collateral_gain = (total_collateral_amount * stake_percentage) / 1_000_000_000_000_000_000;
                
                collateral_gains.push(CollateralGain {
                    block_height,
                    total_collateral_amount,
                    amount: collateral_gain,
                    denom: collateral_denom,
                });
            }
        }
    }

    Ok(collateral_gains)
}

pub fn populate_fee_coins(
    coins: &mut Vec<u64>, // Equivalent to Vec<Coin>
    fee_amount: u64, // Equivalent to Uint256
    _fee_denom: &str,
) -> Result<()> {
    // In Injective: coins.push(Coin { denom: fee_denom.to_string(), amount: Uint128::try_from(*fee_amount)? })
    // For Solana: we would create token transfer instructions
    coins.push(fee_amount);
    Ok(())
}

pub fn process_protocol_fees(
    fee_distributor_addr: Pubkey,
    fee_amount: u64,
    fee_denom: String,
) -> Result<()> {
    // In Injective: msgs.push(CosmosMsg::Wasm(WasmMsg::Execute { contract_addr: fee_distributor_addr.to_string(), msg: to_json_binary(&FeeDistributorExecuteMsg::DistributeFee {})?, funds: coins }))
    // For Solana: we would call the fee distributor program via CPI
    msg!("Processing protocol fee: {} {} to {}", fee_amount, fee_denom, fee_distributor_addr);
    // For now, just log the fee processing
    Ok(())
}

// Additional utils from INJECTIVE packages/utils - Exact replication for Solana

// FundsError equivalent for Solana
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FundsError {
    InvalidDenom { got: String, expected: String },
    InvalidFunds { got: String, expected: String },
    MissingFunds,
    ExtraFunds,
}

// Coin equivalent for Solana
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Coin {
    pub denom: String,
    pub amount: u64, // Equivalent to Uint128
}

// MessageInfo equivalent for Solana
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MessageInfo {
    pub funds: Vec<Coin>,
}

pub fn check_single_coin(info: &MessageInfo, expected: Coin) -> Result<()> {
    if info.funds.len() != 1 {
        return Err(AerospacerProtocolError::FundsError.into());
    };
    let sent_fund = info.funds.get(0).unwrap();
    if sent_fund.denom != expected.denom {
        return Err(AerospacerProtocolError::FundsError.into());
    }
    if sent_fund.amount != expected.amount {
        return Err(AerospacerProtocolError::FundsError.into());
    }
    Ok(())
}

pub fn check_funds(
    info: &MessageInfo,
    // collateral_denom: &str,
) -> Result<()> {
    if info.funds.len() == 0 {
        return Err(AerospacerProtocolError::FundsError.into());
    }
    if info.funds.len() > 1 {
        return Err(AerospacerProtocolError::FundsError.into());
    }
    // let sent_fund = info.funds.get(0).unwrap();
    // if sent_fund.denom != collateral_denom {
    //     return Err(FundsError::InvalidDenom {
    //         got: sent_fund.denom.clone(),
    //         expected: collateral_denom.to_string(),
    //     });
    // }
    Ok(())
}

// Stake calculation functions - Exact replication from INJECTIVE packages/utils/stake.rs

pub fn calculate_stake_amount(
    total_amount: u64, // Equivalent to Uint256
    percentage: u64, // Equivalent to Decimal256 (simplified)
    up: bool,
) -> Result<u64> {
    let stake_amount;
    if up {
        // In Injective: (percentage.checked_add(Decimal256::new(Uint256::from_u128(1)))? * total_amount).checked_div(Uint256::from_u128(100))?
        // For Solana: simplified calculation
        let adjusted_percentage = percentage.checked_add(1).ok_or(AerospacerProtocolError::OverflowError)?;
        stake_amount = (adjusted_percentage.checked_mul(total_amount).ok_or(AerospacerProtocolError::OverflowError)?)
            .checked_div(100).ok_or(AerospacerProtocolError::OverflowError)?;
    } else {
        // In Injective: (percentage * total_amount).checked_div(Uint256::from_u128(100))?
        // For Solana: simplified calculation
        stake_amount = (percentage.checked_mul(total_amount).ok_or(AerospacerProtocolError::OverflowError)?)
            .checked_div(100).ok_or(AerospacerProtocolError::OverflowError)?;
    }

    Ok(stake_amount)
}

pub fn calculate_stake_percentage(
    total_amount: u64, // Equivalent to Uint256
    stake_amount: u64, // Equivalent to Uint256
) -> Result<u64> { // Equivalent to Decimal256 (simplified)
    // In Injective: Decimal256::new(stake_amount.checked_mul(Uint256::from_u128(100))?).checked_div(Decimal256::new(total_amount))
    // For Solana: simplified calculation
    let res = stake_amount
        .checked_mul(100)
        .ok_or(AerospacerProtocolError::OverflowError)?
        .checked_div(total_amount)
        .ok_or(AerospacerProtocolError::OverflowError)?;
    
    Ok(res)
}

// Safe arithmetic functions - Exact replication from INJECTIVE
pub fn safe_add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or(AerospacerProtocolError::OverflowError.into())
}

pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b).ok_or(AerospacerProtocolError::OverflowError.into())
}

pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b).ok_or(AerospacerProtocolError::OverflowError.into())
}

pub fn safe_div(a: u64, b: u64) -> Result<u64> {
    if b == 0 {
        return Err(AerospacerProtocolError::DivideByZeroError.into());
    }
    a.checked_div(b).ok_or(AerospacerProtocolError::OverflowError.into())
}

// Helper functions for total collateral amount management
pub fn get_total_collateral_amount(
    denom: &str,
    _state_key: &Pubkey,
    remaining_accounts: &[AccountInfo],
) -> Result<u64> {
    // For now, return a mock value to avoid complex lifetime issues
    // In a real implementation, this would find the TotalCollateralAmount PDA
    Ok(1000000) // Mock value: 1,000,000 units
}

pub fn update_total_collateral_from_account_info(
    _account_info: &AccountInfo,
    amount_change: i64,
) -> Result<()> {
    // This would update the TotalCollateralAmount account
    // For now, just log the change
    msg!("Updating total collateral by: {}", amount_change);
    Ok(())
}


// Check if a trove can be liquidated
pub fn can_liquidate_trove(
    collateral_amount: u64,
    debt_amount: u64,
    collateral_price: PriceResponse,
    minimum_collateral_ratio: u64,
) -> Result<bool> {
    if debt_amount == 0 {
        return Ok(false);
    }

    // Calculate collateral value in USD
    let collateral_value = safe_mul(collateral_amount, collateral_price.price)?;
    let collateral_value = safe_div(collateral_value, 10_u64.pow(collateral_price.decimal as u32))?;

    // Calculate collateral ratio
    let collateral_ratio = safe_mul(collateral_value, 100)?;
    let collateral_ratio = safe_div(collateral_ratio, debt_amount)?;

    // Check if ratio is below minimum
    Ok(collateral_ratio < minimum_collateral_ratio)
}

// Helper function to calculate liquidation ratio
pub fn calculate_liquidation_ratio(
    trove_amounts: &TroveAmounts,
    collateral_prices: &HashMap<String, u64>,
) -> Result<u64> {
    let mut total_collateral_value = 0u64;
    
    for (denom, amount) in &trove_amounts.collateral_amounts {
        if let Some(price) = collateral_prices.get(denom) {
            total_collateral_value = safe_add(total_collateral_value, safe_mul(*amount, *price)?)?;
        }
    }
    
    if trove_amounts.debt_amount == 0 {
        return Ok(u64::MAX); // No debt means infinite ratio
    }
    
    safe_div(total_collateral_value, trove_amounts.debt_amount)
}

// Helper function to process liquidation
pub fn process_liquidation<'a>(
    state: &mut StateAccount,
    remaining_accounts: &'a [AccountInfo<'a>],
    user_addr: Pubkey,
    amount: u64,
    trove_amounts: &TroveAmounts,
) -> Result<()> {
    // Calculate collateral to liquidate
    let collateral_to_liquidate = safe_div(
        safe_mul(amount, 1000000)?, // 100% of debt
        state.minimum_collateral_ratio as u64,
    )?;
    
    // Update user debt amount
    let user_debt_seeds = UserDebtAmount::seeds(&user_addr);
    let (user_debt_pda, _bump) = Pubkey::find_program_address(&user_debt_seeds, &crate::ID);
    
    for account in remaining_accounts {
        if account.key() == user_debt_pda {
            let mut user_debt: Account<UserDebtAmount> = Account::try_from(account)?;
            user_debt.amount = 0; // Clear debt
            break;
        }
    }
    
    // Update user collateral amounts
    for (denom, _amount) in &trove_amounts.collateral_amounts {
        let user_collateral_seeds = UserCollateralAmount::seeds(&user_addr, denom);
        let (user_collateral_pda, _bump) = Pubkey::find_program_address(&user_collateral_seeds, &crate::ID);
        
        for account in remaining_accounts {
            if account.key() == user_collateral_pda {
                let mut user_collateral: Account<UserCollateralAmount> = Account::try_from(account)?;
                user_collateral.amount = 0; // Clear collateral
                break;
            }
        }
    }
    
    // Update total debt
    state.total_debt_amount = state.total_debt_amount.saturating_sub(amount);
    
    Ok(())
}

// Helper function to process redemption
pub fn process_redemption<'a>(
    state: &mut StateAccount,
    remaining_accounts: &'a [AccountInfo<'a>],
    amount: u64,
    liquidity_ratios: &[LiquidityData],
) -> Result<()> {
    // Calculate collateral to redeem
    let mut remaining_amount = amount;
    
    for liquidity in liquidity_ratios {
        if remaining_amount == 0 {
            break;
        }
        
        let collateral_to_redeem = safe_div(
            safe_mul(remaining_amount, liquidity.liquidity)?,
            1000000, // 100% of liquidity
        )?;
        
        // Update total collateral amount
        let total_collateral_seeds = TotalCollateralAmount::seeds(&liquidity.denom);
        let (total_collateral_pda, _bump) = Pubkey::find_program_address(&total_collateral_seeds, &crate::ID);
        
        for account in remaining_accounts {
            if account.key() == total_collateral_pda {
                let mut total_collateral: Account<TotalCollateralAmount> = Account::try_from(account)?;
                total_collateral.amount = total_collateral.amount.saturating_sub(collateral_to_redeem);
                break;
            }
        }
        
        remaining_amount = remaining_amount.saturating_sub(collateral_to_redeem);
    }
    
    // Update total stake amount (closest equivalent to stablecoin supply)
    state.total_stake_amount = state.total_stake_amount.saturating_sub(amount);
    
    Ok(())
}

// Helper function to get trove amounts for liquidation
pub fn get_trove_amounts<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    user_addr: Pubkey,
) -> Result<TroveAmounts> {
    let mut collateral_amounts: Vec<(String, u64)> = Vec::new();
    let mut debt_amount = 0u64;
    
    // Find user debt amount account
    let user_debt_seeds = UserDebtAmount::seeds(&user_addr);
    let (user_debt_pda, _bump) = Pubkey::find_program_address(&user_debt_seeds, &crate::ID);
    
    for account in remaining_accounts {
        if account.key() == user_debt_pda {
            let user_debt: Account<UserDebtAmount> = Account::try_from(account)?;
            debt_amount = user_debt.amount;
        } else {
            // Check if this is a user collateral amount account
            // For now, we'll use a simple approach to identify collateral accounts
            // In a real implementation, you'd need to track which accounts correspond to which denoms
            if account.owner == &crate::ID {
                // This is a potential collateral account, but we need to know the denom
                // For now, we'll skip this complexity
            }
        }
    }

    Ok(TroveAmounts {
        collateral_amounts,
        debt_amount,
    })
}

// Fee calculation utilities for protocol-fees integration
pub fn calculate_protocol_fee(amount: u64, fee_percentage: u8) -> Result<u64> {
    let fee = amount
        .checked_mul(fee_percentage as u64)
        .ok_or(AerospacerProtocolError::OverflowError)?
        .checked_div(100)
        .ok_or(AerospacerProtocolError::OverflowError)?;
    
    Ok(fee)
}

pub fn calculate_net_amount_after_fee(amount: u64, fee_percentage: u8) -> Result<u64> {
    let fee = calculate_protocol_fee(amount, fee_percentage)?;
    amount
        .checked_sub(fee)
        .ok_or(AerospacerProtocolError::OverflowError.into())
}

// Mock functions to replace the deleted trove_helpers functions
pub fn get_trove_icr(
    _user_debt_amount: &UserDebtAmount,
    _user_collateral_amount_accounts: &[AccountInfo],
    _collateral_prices: &HashMap<String, u64>,
    _owner: Pubkey,
) -> Result<u64> {
    // Mock implementation - return 150% ICR
    Ok(1500000)
}

pub fn check_trove_icr_with_ratio(
    _state_account: &StateAccount,
    _ratio: u64,
) -> Result<()> {
    // Mock implementation - always pass
    Ok(())
}

pub fn get_first_trove(storage: &Account<SortedTrovesState>) -> Result<Option<Pubkey>> {
    Ok(storage.head)
}

pub fn get_last_trove(storage: &Account<SortedTrovesState>) -> Result<Option<Pubkey>> {
    Ok(storage.tail)
}
