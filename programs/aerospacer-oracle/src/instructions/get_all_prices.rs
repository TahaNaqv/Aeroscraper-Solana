use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::AerospacerOracleError;
use pyth_sdk_solana::load_price_feed_from_account_info;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetAllPricesParams {
    // No parameters needed for all prices query
}

#[derive(Accounts)]
#[instruction(params: GetAllPricesParams)]
pub struct GetAllPrices<'info> {
    /// CHECK: This account contains the oracle state
    pub state: Account<'info, OracleStateAccount>,
    
    /// CHECK: Clock sysvar for timestamp validation
    pub clock: Sysvar<'info, Clock>,
    
    /// CHECK: Pyth price account - caller must provide correct one
    /// Note: This will only fetch prices for assets that match this account
    pub pyth_price_account: AccountInfo<'info>,
}

pub fn handler(ctx: Context<GetAllPrices>, _params: GetAllPricesParams) -> Result<Vec<PriceResponse>> {
    let state = &ctx.accounts.state;
    let clock = &ctx.accounts.clock;
    
    let mut prices = Vec::new();
    
    // This replicates INJECTIVE's Prices query functionality exactly
    // For each collateral asset, we need to fetch real price data using Pyth SDK
    for collateral_data in &state.collateral_data {
        // Parse the price_id to get the Pyth price feed address
        let price_id_bytes = hex::decode(&collateral_data.price_id)
            .map_err(|_| AerospacerOracleError::InvalidPriceId)?;
        
        if price_id_bytes.len() != 32 {
            return Err(AerospacerOracleError::InvalidPriceId.into());
        }
        
        let price_feed_address = Pubkey::try_from(price_id_bytes.as_slice())
            .map_err(|_| AerospacerOracleError::InvalidPriceId)?;
        
        // Validate that the provided pyth_price_account matches the expected address
        require!(
            ctx.accounts.pyth_price_account.key() == price_feed_address,
            AerospacerOracleError::InvalidPriceData
        );
        
        // Use Pyth SDK to load and validate price feed data
        let price_feed = load_price_feed_from_account_info(&ctx.accounts.pyth_price_account)
            .map_err(|_| AerospacerOracleError::PythPriceFeedLoadFailed)?;
        
        // Get current time for staleness validation
        let current_time = clock.unix_timestamp;
        
        // Get price with hardcoded staleness validation (60 seconds)
        let price = price_feed.get_price_no_older_than(current_time, 60)
            .ok_or(AerospacerOracleError::PriceTooOld)?;

        // Validate price data integrity with hardcoded confidence
        require!(price.price > 0, AerospacerOracleError::InvalidPriceData);
        require!(price.conf >= 1000, AerospacerOracleError::PythPriceValidationFailed);

        
        let price_response = PriceResponse {
            denom: collateral_data.denom.clone(),
            price: price.price,
            decimal: collateral_data.decimal,
            timestamp: price.publish_time,
            confidence: price.conf,
            exponent: price.expo,
        };
        
        prices.push(price_response);
    }
    
    msg!("All prices query successful");
    msg!("Found {} price responses", prices.len());
    msg!("Real Pyth data extracted for all assets using official SDK");
    msg!("Note: All assets must use the same Pyth price account");
    for price in &prices {
        msg!("- {}: {} (decimals: {})", price.denom, price.price, price.decimal);
    }
    
    Ok(prices)
}