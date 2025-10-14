use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::AerospacerOracleError;
use pyth_sdk_solana::state::SolanaPriceAccount;

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
}

pub fn handler(ctx: Context<GetAllPrices>, _params: GetAllPricesParams) -> Result<Vec<PriceResponse>> {
    let state = &ctx.accounts.state;
    let clock = &ctx.accounts.clock;
    
    // Get remaining accounts (should contain Pyth price accounts for each asset)
    let remaining_accounts = &ctx.remaining_accounts;
    
    // Validate we have enough Pyth accounts for all assets
    require!(
        remaining_accounts.len() >= state.collateral_data.len(),
        AerospacerOracleError::InvalidPriceData
    );
    
    let mut prices = Vec::new();
    
    // THIS CODE IS FOR TESTING PURPOSES ONLY
    // UNCOMMENT DURING TESTING
    // For each collateral asset, return mock price data for testing
    // for collateral_data in &state.collateral_data {
    //     let mock_price = match collateral_data.denom.as_str() {
    //         "SOL" => PriceResponse {
    //             denom: collateral_data.denom.clone(),
    //             price: 183415750000, // Mock SOL price: $183.41
    //             decimal: collateral_data.decimal,
    //             timestamp: clock.unix_timestamp,
    //             confidence: 1000,
    //             exponent: -9,
    //         },
    //         "ETH" => PriceResponse {
    //             denom: collateral_data.denom.clone(),
    //             price: 7891575000, // Mock ETH price: $7,891.58
    //             decimal: collateral_data.decimal,
    //             timestamp: clock.unix_timestamp,
    //             confidence: 1000,
    //             exponent: -6,
    //         },
    //         "BTC" => PriceResponse {
    //             denom: collateral_data.denom.clone(),
    //             price: 125000000000, // Mock BTC price: $125,000.00
    //             decimal: collateral_data.decimal,
    //             timestamp: clock.unix_timestamp,
    //             confidence: 1000,
    //             exponent: -8,
    //         },
    //         _ => PriceResponse {
    //             denom: collateral_data.denom.clone(),
    //             price: 1000000000, // Mock price: $1.00
    //             decimal: collateral_data.decimal,
    //             timestamp: clock.unix_timestamp,
    //             confidence: 1000,
    //             exponent: -6,
    //         }
    //     };
        
    //     prices.push(mock_price);
    // }
    
    // msg!("All prices query successful (TESTING MODE)");
    // msg!("Found {} price responses", prices.len());
    // msg!("Mock price data returned for testing purposes");
    // msg!("Note: Pyth integration is commented out for testing");
    // for price in &prices {
    //     msg!("- {}: {} (decimals: {})", price.denom, price.price, price.decimal);
    // }
    
    // Ok(prices)

    // PRODUCTION PYTH INTEGRATION CODE (COMMENT FOR TESTING)
    // This replicates INJECTIVE's Prices query functionality exactly
    // For each collateral asset, fetch real price data using corresponding Pyth account
    for (index, collateral_data) in state.collateral_data.iter().enumerate() {
        // Get the corresponding Pyth price account from remaining_accounts
        let pyth_price_account = &remaining_accounts[index];
        
        // Parse the price_id to get the expected Pyth price feed address
        let price_id_bytes = hex::decode(&collateral_data.price_id)
            .map_err(|_| AerospacerOracleError::InvalidPriceId)?;
        
        if price_id_bytes.len() != 32 {
            return Err(AerospacerOracleError::InvalidPriceId.into());
        }
        
        let price_feed_address = Pubkey::try_from(price_id_bytes.as_slice())
            .map_err(|_| AerospacerOracleError::InvalidPriceId)?;
        
        // Validate that the provided pyth_price_account matches the expected address
        require!(
            pyth_price_account.key() == price_feed_address,
            AerospacerOracleError::InvalidPriceData
        );
        
        // Use Pyth SDK to load and validate price feed data (reusing get_price logic)
        let price_feed = SolanaPriceAccount::account_info_to_feed(pyth_price_account)
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
    msg!("Each asset uses its own Pyth price account via remaining_accounts");
    for price in &prices {
        msg!("- {}: {} ± {} x 10^{}", price.denom, price.price, price.confidence, price.exponent);
    }
    
    Ok(prices)
}