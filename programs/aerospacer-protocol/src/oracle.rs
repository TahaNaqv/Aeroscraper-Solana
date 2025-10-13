use anchor_lang::prelude::*;
use anchor_lang::solana_program::{hash::hash, instruction::{Instruction, AccountMeta}};
use crate::error::*;

/// Oracle integration for price feeds
/// This module provides clean integration with our aerospacer-oracle contract

/// Price data structure (matches aerospacer-oracle PriceResponse)
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
pub struct OracleContext<'info> {
    /// Our oracle program
    pub oracle_program: AccountInfo<'info>,
    
    /// Oracle state account
    pub oracle_state: AccountInfo<'info>,
    
    /// Pyth price account for the collateral asset
    pub pyth_price_account: AccountInfo<'info>,
    
    /// Clock sysvar
    pub clock: AccountInfo<'info>,
}

/// Oracle integration implementation
impl<'info> OracleContext<'info> {
    /// Get price for a specific collateral denom via CPI to our oracle
    pub fn get_price(&self, denom: &str) -> Result<PriceData> {
        // Build the CPI instruction to call oracle's get_price
        let price_response = get_price_via_cpi(
            denom.to_string(),
            self.oracle_program.to_account_info(),
            self.oracle_state.to_account_info(),
            self.pyth_price_account.to_account_info(),
            self.clock.to_account_info(),
        )?;
        
        // Convert PriceResponse to PriceData
        Ok(PriceData {
            denom: price_response.denom,
            price: price_response.price,
            decimal: price_response.decimal,
            confidence: price_response.confidence,
            timestamp: price_response.timestamp,
            exponent: price_response.exponent,
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
/// 
/// ICR Convention:
/// All ICR values are represented as simple percentages (not scaled).
/// Example: 150% ICR = 150, 200% ICR = 200
/// This avoids u64 overflow issues while maintaining sufficient precision
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
    
    /// Calculate collateral ratio as a percentage (100 = 100%)
    /// Returns ICR as an unscaled percentage for comparison
    /// Example: 150% ICR = 150
    pub fn calculate_collateral_ratio(
        collateral_value: u64,
        debt_amount: u64,
    ) -> Result<u64> {
        if debt_amount == 0 {
            return Ok(u64::MAX);
        }
        
        // Calculate ratio as percentage (multiply by 100)
        // Use u128 to prevent overflow during calculation
        let ratio = (collateral_value as u128)
            .checked_mul(100)
            .ok_or(AerospacerProtocolError::OverflowError)?
            .checked_div(debt_amount as u128)
            .ok_or(AerospacerProtocolError::OverflowError)?;
        
        // Convert back to u64, should be safe as percentage ratios are small
        u64::try_from(ratio).map_err(|_| AerospacerProtocolError::OverflowError.into())
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
    
    /// Calculate total collateral value across multiple denoms
    /// Used for multi-collateral trove ICR calculation
    pub fn calculate_multi_collateral_value(
        collateral_amounts: &[(String, u64)],
        prices: &[(String, u64, u8)], // (denom, price, decimal)
    ) -> Result<u64> {
        let mut total_value = 0u64;
        
        for (denom, amount) in collateral_amounts {
            // Find matching price data
            let price_data = prices.iter()
                .find(|(d, _, _)| d == denom)
                .ok_or(AerospacerProtocolError::InvalidAmount)?;
            
            let value = Self::calculate_collateral_value(
                *amount,
                price_data.1,
                price_data.2,
            )?;
            
            total_value = total_value
                .checked_add(value)
                .ok_or(AerospacerProtocolError::OverflowError)?;
        }
        
        Ok(total_value)
    }
    
    /// Calculate ICR for a trove with multiple collateral types
    pub fn calculate_trove_icr(
        collateral_amounts: &[(String, u64)],
        debt_amount: u64,
        prices: &[(String, u64, u8)],
    ) -> Result<u64> {
        if debt_amount == 0 {
            return Ok(u64::MAX);
        }
        
        let total_collateral_value = Self::calculate_multi_collateral_value(
            collateral_amounts,
            prices,
        )?;
        
        Self::calculate_collateral_ratio(total_collateral_value, debt_amount)
    }
}

/// PriceResponse struct (matches oracle contract's return type)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceResponse {
    pub denom: String,
    pub price: i64,
    pub decimal: u8,
    pub timestamp: i64,
    pub confidence: u64,
    pub exponent: i32,
}

/// Execute CPI call to oracle contract's get_price instruction
pub fn get_price_via_cpi<'info>(
    denom: String,
    oracle_program: AccountInfo<'info>,
    oracle_state: AccountInfo<'info>,
    pyth_price_account: AccountInfo<'info>,
    clock: AccountInfo<'info>,
) -> Result<PriceResponse> {
    // Calculate discriminator for get_price instruction
    // Anchor uses: SHA256("global:get_price")[0..8]
    let preimage = b"global:get_price";
    let hash_result = hash(preimage);
    let discriminator = &hash_result.to_bytes()[..8];
    
    // Serialize the GetPriceParams { denom }
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(discriminator);
    
    // Serialize params struct: { denom: String }
    denom.serialize(&mut instruction_data)?;
    
    // Build account metas for CPI (include all accounts including program)
    let account_metas = vec![
        AccountMeta::new(oracle_state.key(), false),
        AccountMeta::new_readonly(pyth_price_account.key(), false),
        AccountMeta::new_readonly(clock.key(), false),
    ];
    
    // Build the instruction
    let ix = Instruction {
        program_id: oracle_program.key(),
        accounts: account_metas,
        data: instruction_data,
    };
    
    // Execute CPI (data accounts + program)
    // Note: Account metas only include data accounts, but invoke needs the program too
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            oracle_state.clone(),
            pyth_price_account.clone(),
            clock.clone(),
            oracle_program.clone(),
        ],
    )?;
    
    msg!("Oracle CPI executed successfully for denom: {}", denom);
    
    // Parse return data from oracle program
    let return_data = anchor_lang::solana_program::program::get_return_data()
        .ok_or(AerospacerProtocolError::InvalidAmount)?;
    
    // Verify the return data is from our oracle program
    require!(
        return_data.0 == oracle_program.key(),
        AerospacerProtocolError::InvalidAmount
    );
    
    // Deserialize PriceResponse
    let price_response: PriceResponse = PriceResponse::deserialize(&mut &return_data.1[..])?;
    
    msg!("Price received: {} for {}", price_response.price, price_response.denom);
    
    Ok(price_response)
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
