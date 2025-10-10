use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

/// Oracle integration for price feeds
/// This module provides clean integration with our aerospacer-oracle contract

/// Price data structure (matches aerospacer-oracle)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceData {
    pub denom: String,
    pub price: i64, // Oracle returns i64
    pub decimal: u8,
    pub confidence: u64,
    pub timestamp: i64,
    pub exponent: i32,
}

/// Oracle context for price queries via CPI
#[derive(Accounts)]
pub struct OracleContext<'info> {
    /// CHECK: Our oracle program
    #[account(mut)]
    pub oracle_program: AccountInfo<'info>,
    
    /// CHECK: Oracle state account
    #[account(mut)]
    pub oracle_state: AccountInfo<'info>,
}

/// Oracle integration implementation
impl<'info> OracleContext<'info> {
    /// Get price for a specific collateral denom via CPI to our oracle
    pub fn get_price(&self, denom: &str) -> Result<PriceData> {
        // For now, return mock prices based on denom
        // TODO: Implement real CPI call to aerospacer-oracle
        // Example CPI call would be:
        // let cpi_accounts = GetPrice {
        //     state: self.oracle_state.to_account_info(),
        //     pyth_price_account: pyth_account.to_account_info(),
        //     clock: Clock::get()?.to_account_info(),
        // };
        // let cpi_program = self.oracle_program.to_account_info();
        // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // aerospacer_oracle::cpi::get_price(cpi_ctx, GetPriceParams { denom: denom.to_string() })
        
        let (price, decimal) = match denom {
            "SOL" => (100_000_000i64, 9), // $100 with 9 decimals
            "USDC" => (1_000_000i64, 6),  // $1 with 6 decimals
            "INJ" => (144015750000i64, 18), // Mock INJ price with 18 decimals
            "ATOM" => (6313260000i64, 6),  // Mock ATOM price with 6 decimals
            _ => (50_000_000i64, 6),      // $50 with 6 decimals
        };
        
        Ok(PriceData {
            denom: denom.to_string(),
            price: price as i64, // Convert to i64 for oracle compatibility
            decimal,
            confidence: 1000, // Mock confidence
            timestamp: Clock::get()?.unix_timestamp,
            exponent: -8, // Mock exponent
        })
    }
    
    /// Get prices for all supported collateral denoms via CPI
    pub fn get_all_prices(&self) -> Result<Vec<PriceData>> {
        let denoms = vec!["SOL", "USDC", "INJ", "ATOM"];
        let mut prices = Vec::new();
        
        for denom in denoms {
            let price_data = self.get_price(denom)?;
            prices.push(price_data);
        }
        
        Ok(prices)
    }
    
    /// Validate price data
    pub fn validate_price(&self, price_data: &PriceData) -> Result<()> {
        // Check if price is within reasonable bounds
        require!(
            price_data.price > 0,
            AerospacerProtocolError::InvalidAmount
        );
        
        // Check if price is not too old (within 5 minutes)
        let current_time = Clock::get()?.unix_timestamp;
        let max_age = 300; // 5 minutes in seconds
        
        require!(
            current_time - price_data.timestamp <= max_age,
            AerospacerProtocolError::InvalidAmount
        );
        
        Ok(())
    }
}

/// Price calculation utilities
pub struct PriceCalculator;

impl PriceCalculator {
    /// Calculate collateral value in USD
    pub fn calculate_collateral_value(
        amount: u64,
        price: u64,
        decimal: u8,
    ) -> Result<u64> {
        let decimal_factor = 10_u64.pow(decimal as u32);
        let value = amount
            .checked_mul(price)
            .ok_or(AerospacerProtocolError::OverflowError)?
            .checked_div(decimal_factor)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        Ok(value)
    }
    
    /// Calculate collateral ratio
    pub fn calculate_collateral_ratio(
        collateral_value: u64,
        debt_amount: u64,
    ) -> Result<u64> {
        if debt_amount == 0 {
            return Ok(u64::MAX);
        }
        
        let ratio = collateral_value
            .checked_mul(100)
            .ok_or(AerospacerProtocolError::OverflowError)?
            .checked_div(debt_amount)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        Ok(ratio)
    }
    
    /// Check if trove is liquidatable
    pub fn is_liquidatable(
        collateral_value: u64,
        debt_amount: u64,
        minimum_ratio: u64,
    ) -> Result<bool> {
        if debt_amount == 0 {
            return Ok(false);
        }
        
        let ratio = Self::calculate_collateral_ratio(collateral_value, debt_amount)?;
        Ok(ratio < minimum_ratio)
    }
}

/// Mock oracle for testing
pub struct MockOracle;

impl MockOracle {
    /// Get mock price data
    pub fn get_mock_price(denom: &str) -> PriceData {
        let (price, decimal) = match denom {
            "SOL" => (100_000_000i64, 9),
            "USDC" => (1_000_000i64, 6),
            "INJ" => (144015750000i64, 18),
            "ATOM" => (6313260000i64, 6),
            _ => (50_000_000i64, 6),
        };
        
        PriceData {
            denom: denom.to_string(),
            price: price as i64, // Convert to i64 for oracle compatibility
            decimal,
            confidence: 1000,
            timestamp: Clock::get().unwrap().unix_timestamp,
            exponent: -8, // Mock exponent
        }
    }
    
    /// Get all mock prices
    pub fn get_all_mock_prices() -> Vec<PriceData> {
        let denoms = vec!["SOL", "USDC", "INJ", "ATOM"];
        denoms.into_iter()
            .map(|denom| Self::get_mock_price(denom))
            .collect()
    }
}
