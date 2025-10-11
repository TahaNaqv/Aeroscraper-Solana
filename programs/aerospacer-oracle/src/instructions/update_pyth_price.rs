use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::AerospacerOracleError;
use pyth_sdk_solana::load_price_feed_from_account_info;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdatePythPriceParams {
    /// Asset denomination to update price for
    pub denom: String,
}

#[derive(Accounts)]
#[instruction(params: UpdatePythPriceParams)]
pub struct UpdatePythPrice<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = state.admin == admin.key() @ AerospacerOracleError::Unauthorized
    )]
    pub state: Account<'info, OracleStateAccount>,
    
    /// CHECK: Pyth price account to update from
    pub pyth_price_account: AccountInfo<'info>,
    
    /// CHECK: Clock sysvar for timestamp validation
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<UpdatePythPrice>, params: UpdatePythPriceParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let clock = &ctx.accounts.clock;
    
    // Find the collateral data for the requested denom
    let collateral_data = state.collateral_data
        .iter_mut()
        .find(|d| d.denom == params.denom)
        .ok_or(AerospacerOracleError::PriceFeedNotFound)?;
    
    // THIS CODE IS FOR TESTING PURPOSES ONLY
    // UNCOMMENT DURING TESTING
    // For testing, we just update the timestamp without fetching real Pyth data
    // msg!("Pyth price update successful (TESTING MODE)");
    // msg!("Denom: {}", params.denom);
    // msg!("Mock price update completed for testing");
    // msg!("Note: Pyth integration is commented out for testing");
    
    // state.last_update = clock.unix_timestamp;
    
    // msg!("Updated at: {}", clock.unix_timestamp);
    
    // Ok(())

    // PRODUCTION PYTH INTEGRATION CODE (COMMENTED OUT FOR TESTING)
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
    
    // Get latest price with hardcoded staleness validation (60 seconds)
    let price = price_feed.get_price_no_older_than(current_time, 60)
        .ok_or(AerospacerOracleError::PriceTooOld)?;

    // Validate price data integrity with hardcoded confidence
    require!(price.price > 0, AerospacerOracleError::PythPriceValidationFailed);
    require!(price.conf >= 1000, AerospacerOracleError::PythPriceValidationFailed);

    
    // Update the last update timestamp
    state.last_update = clock.unix_timestamp;
    
    msg!("Pyth price update successful");
    msg!("Denom: {}", params.denom);
    msg!("New Price: {} ± {} x 10^{}", price.price, price.conf, price.expo);
    msg!("Publish Time: {}", price.publish_time);
    msg!("Updated at: {}", clock.unix_timestamp);
    
    Ok(())
}