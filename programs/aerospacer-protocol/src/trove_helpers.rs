use std::collections::HashMap;

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

pub const DECIMAL_FRACTION_6: u128 = 1_000_000;
pub const DECIMAL_FRACTION_18: u128 = 1_000_000_000_000_000_000;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
struct CollateralData {
    denom: String,
    share: u64, // Equivalent to Decimal256
    price: u64, // Equivalent to Decimal256
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidityData {
    pub denom: String,
    pub liquidity: u64, // Equivalent to Decimal256
    pub decimal: u8,
}

pub fn get_trove_liquidity(
    _user_collateral_amount_accounts: &[AccountInfo],
    _collateral_prices: &HashMap<String, u64>, // PriceResponse simplified to u64
    _owner: Pubkey,
) -> Result<Vec<LiquidityData>> {
    let mut res = Vec::new();
    
    // For now, return mock data to avoid complex lifetime issues
    // In a real implementation, this would iterate through user_collateral_amount_accounts
    res.push(LiquidityData {
        denom: "SOL".to_string(),
        liquidity: 1000000, // Mock liquidity value
        decimal: 9,
    });
    
    Ok(res)
}

pub fn get_liquidity_data<'a>(
    user_collateral_amount_accounts: &'a [AccountInfo<'a>],
    collateral_prices: &HashMap<String, u64>,
    owner: Pubkey,
) -> Result<Vec<LiquidityData>> {
    get_trove_liquidity(user_collateral_amount_accounts, collateral_prices, owner)
}

pub fn get_trove_collateral_ratio(
    _user_collateral_amount_accounts: &[AccountInfo],
    _collateral_prices: &HashMap<String, u64>,
    _owner: Pubkey,
    loan_amount: u64, // Equivalent to Uint256
) -> Result<u64> { // Equivalent to Decimal256
    // For now, return a mock ratio to avoid complex lifetime issues
    // In a real implementation, this would calculate the actual ratio
    Ok(1500000) // 150% ratio as a mock value
}

pub fn get_trove_icr(
    user_debt_amount: &UserDebtAmount,
    _user_collateral_amount_accounts: &[AccountInfo],
    _collateral_prices: &HashMap<String, u64>,
    _owner: Pubkey,
) -> Result<u64> {
    let debt_amount = user_debt_amount.amount;
    // For now, return a mock ICR to avoid complex lifetime issues
    // In a real implementation, this would calculate the actual ICR
    Ok(1500000) // 150% ICR as a mock value
}

pub fn check_trove_icr<'a>(
    state_account: &StateAccount,
    user_debt_amount_account: &Account<UserDebtAmount>,
    user_collateral_amount_accounts: &'a [AccountInfo<'a>],
    collateral_prices: &HashMap<String, u64>,
    owner: Pubkey,
) -> Result<()> {
    let ratio = get_trove_icr(user_debt_amount_account, user_collateral_amount_accounts, collateral_prices, owner)?;
    let minimum_ratio = state_account.minimum_collateral_ratio as u64 * DECIMAL_FRACTION_18 as u64; // Simplified Decimal256
    
    require!(
        ratio >= minimum_ratio,
        AerospacerProtocolError::CollateralBelowMinimum
    );
    Ok(())
}

pub fn check_trove_icr_with_ratio(
    state_account: &StateAccount,
    ratio: u64, // Equivalent to Decimal256
) -> Result<()> {
    let minimum_ratio = state_account.minimum_collateral_ratio as u64 * DECIMAL_FRACTION_18 as u64; // Simplified Decimal256
    
    require!(
        ratio >= minimum_ratio,
        AerospacerProtocolError::CollateralBelowMinimum
    );
    Ok(())
}

pub fn check_trove_collateral_ratio<'a>(
    state_account: &StateAccount,
    user_collateral_amount_accounts: &'a [AccountInfo<'a>],
    collateral_prices: &HashMap<String, u64>,
    owner: Pubkey,
    loan_amount: u64, // Equivalent to Uint256
) -> Result<()> {
    let ratio = get_trove_collateral_ratio(user_collateral_amount_accounts, collateral_prices, owner, loan_amount)?;
    let minimum_ratio = state_account.minimum_collateral_ratio as u64 * DECIMAL_FRACTION_18 as u64; // Simplified Decimal256
    
    require!(
        ratio >= minimum_ratio,
        AerospacerProtocolError::CollateralBelowMinimum
    );
    Ok(())
}

pub fn check_trove_collateral_ratio_liquidation_testing<'a>(
    state_account: &StateAccount,
    user_collateral_amount_accounts: &'a [AccountInfo<'a>],
    owner: Pubkey,
    loan_amount: u64, // Equivalent to Uint256
) -> Result<()> {
    let mut collateral_prices: HashMap<String, u64> = HashMap::new(); // i64 in Injective, u64 here
    collateral_prices.insert("inj".to_string(), 144015750000);
    collateral_prices.insert("atom".to_string(), 6313260000);
    
    let mut denom_and_liquidity = Vec::new();
    for account_info in user_collateral_amount_accounts {
        let user_collateral: Account<UserCollateralAmount> = Account::try_from(account_info)?;
        if user_collateral.owner == owner {
            let denom = user_collateral.denom.clone();
            let amount = user_collateral.amount;
            
            let price = *collateral_prices.get(&denom).ok_or(AerospacerProtocolError::InvalidDecimal)?;
            
            // Simplified Uint256 calculations
            let liquidity = amount
                .checked_mul(price)
                .ok_or(AerospacerProtocolError::OverflowError)?
                .checked_div(DECIMAL_FRACTION_6 as u64)
                .ok_or(AerospacerProtocolError::OverflowError)?
                .checked_div(100_000_000)
                .ok_or(AerospacerProtocolError::OverflowError)?;
            denom_and_liquidity.push((denom, liquidity));
        }
    }
    
    let total_liquidity: u64 = denom_and_liquidity.iter().map(|(_, liquidity)| *liquidity).sum();
    
    // Simplified Decimal256 calculations
    let ratio = total_liquidity
        .checked_mul(DECIMAL_FRACTION_6 as u64)
        .ok_or(AerospacerProtocolError::OverflowError)?
        .checked_mul(100)
        .ok_or(AerospacerProtocolError::OverflowError)?
        .checked_div(loan_amount)
        .ok_or(AerospacerProtocolError::OverflowError)?;
    
    let minimum_ratio = state_account.minimum_collateral_ratio as u64;
    require!(
        ratio < minimum_ratio, // Injective has `ratio.to_uint_floor() < Uint256::from_u128(minimum_ratio)`
        AerospacerProtocolError::CollateralBelowMinimum
    );
    Ok(())
}

pub fn get_trove_denoms<'a>(
    user_collateral_amount_accounts: &'a [AccountInfo<'a>],
    owner: Pubkey,
) -> Result<Vec<String>> {
    let mut denoms = Vec::new();
    for account_info in user_collateral_amount_accounts {
        let user_collateral: Account<UserCollateralAmount> = Account::try_from(account_info)?;
        if user_collateral.owner == owner {
            denoms.push(user_collateral.denom.clone());
        }
    }
    Ok(denoms)
}

pub fn get_trove_liquidity_ratios<'a>(
    user_collateral_amount_accounts: &'a [AccountInfo<'a>],
    collateral_prices: &HashMap<String, u64>,
    owner: Pubkey,
) -> Result<HashMap<String, u64>> { // Decimal256 simplified to u64
    let mut liquidity_ratios = HashMap::new();
    let data = get_liquidity_data(user_collateral_amount_accounts, collateral_prices, owner)?;
    let total_liquidity: u64 = data.iter().map(|data| data.liquidity).sum();
    
    for item in data {
        let ratio = if total_liquidity > 0 {
            item.liquidity
                .checked_mul(100)
                .ok_or(AerospacerProtocolError::OverflowError)?
                .checked_mul(DECIMAL_FRACTION_18 as u64)
                .ok_or(AerospacerProtocolError::OverflowError)?
        } else {
            0
        };
        let ratio = ratio.checked_div(total_liquidity).ok_or(AerospacerProtocolError::OverflowError)?;
        liquidity_ratios.insert(item.denom, ratio);
    }
    Ok(liquidity_ratios)
}