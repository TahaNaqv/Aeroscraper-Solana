use anchor_lang::prelude::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetPriceParams {
    pub denom: String,
}

#[derive(Accounts)]
#[instruction(params: GetPriceParams)]
pub struct GetPrice<'info> {
    pub state: Account<'info, OracleStateAccount>,
    
    /// CHECK: This is the Pyth price feed account
    pub pyth_price_feed: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<GetPrice>, params: GetPriceParams) -> Result<PriceResponse> {
    let state = &ctx.accounts.state;
    
    // Find the collateral data for the requested denom
    let collateral_data = state.collateral_data
        .iter()
        .find(|d| d.denom == params.denom)
        .ok_or(ErrorCode::CollateralDataNotFound)?;
    
    // TODO: Implement Pyth Network price query
    // In a full implementation, you would:
    // 1. Parse the price_id to get the Pyth price feed address
    // 2. Query the Pyth price feed account
    // 3. Extract price, confidence, and timestamp
    // 4. Return the price response
    
    // Placeholder implementation
    let current_timestamp = Clock::get()?.unix_timestamp;
    
    msg!("Price query successful");
    msg!("Denom: {}", params.denom);
    msg!("Price ID: {}", collateral_data.price_id);
    msg!("Decimal: {}", collateral_data.decimal);
    msg!("Timestamp: {}", current_timestamp);
    
    // Return placeholder price response
    Ok(PriceResponse {
        denom: params.denom,
        price: 1000000, // Placeholder price (1.00 with 6 decimals)
        decimal: collateral_data.decimal,
        timestamp: current_timestamp,
    })
}

#[error_code]
pub enum ErrorCode {
    #[msg("Collateral data not found")]
    CollateralDataNotFound,
    #[msg("Price query failed")]
    PriceQueryFailed,
}
