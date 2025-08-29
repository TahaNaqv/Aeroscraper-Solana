use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::AerospacerOracleError;
use pyth_sdk_solana::load_price_feed_from_account_info;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetPriceParams {
    pub denom: String,
}

#[derive(Accounts)]
#[instruction(params: GetPriceParams)]
pub struct GetPrice<'info> {
    #[account(mut)]
    pub state: Account<'info, OracleStateAccount>,
    
    /// CHECK: This is the Pyth price account that contains the price data
    pub pyth_price_account: AccountInfo<'info>,
    
    /// CHECK: Clock sysvar for timestamp validation
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<GetPrice>, params: GetPriceParams) -> Result<PriceResponse> {
    let state = &ctx.accounts.state;
    let clock = &ctx.accounts.clock;
    
    // Find the collateral data for the requested denom
    let collateral_data = state.collateral_data
        .iter()
        .find(|d| d.denom == params.denom)
        .ok_or(AerospacerOracleError::PriceFeedNotFound)?;
    
    // THIS CODE IS FOR TESTING PURPOSES ONLY
    // UNCOMMENT DURING TESTING
    let d = params.denom.as_str();
    return match d {
        "SOL" => Ok(PriceResponse {
            denom: params.denom,
            price: 183415750000, // Mock SOL price: $183.41
            decimal: collateral_data.decimal,
            timestamp: clock.unix_timestamp,
            confidence: 1000,
            exponent: -9,
        }),
        "ETH" => Ok(PriceResponse {
            denom: params.denom,
            price: 7891575000, // Mock ETH price: $7,891.58
            decimal: collateral_data.decimal,
            timestamp: clock.unix_timestamp,
            confidence: 1000,
            exponent: -6,
        }),
        "BTC" => Ok(PriceResponse {
            denom: params.denom,
            price: 125000000000, // Mock BTC price: $125,000.00
            decimal: collateral_data.decimal,
            timestamp: clock.unix_timestamp,
            confidence: 1000,
            exponent: -8,
        }),
        _ => {
            // For other assets, use a default mock price
            Ok(PriceResponse {
                denom: params.denom,
                price: 1000000000, // Mock price: $1.00
                decimal: collateral_data.decimal,
                timestamp: clock.unix_timestamp,
                confidence: 1000,
                exponent: -6,
            })
        }
    };

    // PRODUCTION PYTH INTEGRATION CODE (COMMENTED OUT FOR TESTING)
    /*
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
    
    msg!("Price query successful");
    msg!("Denom: {}", params.denom);
    msg!("Price ID: {}", collateral_data.price_id);
    msg!("Decimal: {}", collateral_data.decimal);
    msg!("Publish Time: {}", price.publish_time);
    msg!("Price: {} ± {} x 10^{}", price.price, price.conf, price.expo);
    msg!("Real Pyth data extracted successfully using official SDK");
    
    // Return price response with validated Pyth data
    Ok(PriceResponse {
        denom: params.denom,
        price: price.price,
        decimal: collateral_data.decimal,
        timestamp: price.publish_time,
        confidence: price.conf,
        exponent: price.expo,
    })
    */
}