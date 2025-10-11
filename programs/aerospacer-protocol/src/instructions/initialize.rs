use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeParams {
    pub stable_coin_code_id: u64,
    pub oracle_helper_addr: Pubkey,
    pub fee_distributor_addr: Pubkey, // aerospacer-fees program ID
}

#[derive(Accounts)]
#[instruction(params: InitializeParams)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + StateAccount::LEN)]
    pub state: Account<'info, StateAccount>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// CHECK: This is the stable coin mint account
    pub stable_coin_mint: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // Initialize state exactly like INJECTIVE's instantiate
    state.admin = ctx.accounts.admin.key();
    state.stable_coin_addr = ctx.accounts.stable_coin_mint.key();
    state.oracle_helper_addr = params.oracle_helper_addr;
    state.fee_distributor_addr = params.fee_distributor_addr; // This is the aerospacer-fees program ID
    state.minimum_collateral_ratio = DEFAULT_MINIMUM_COLLATERAL_RATIO; // 115%
    state.protocol_fee = DEFAULT_PROTOCOL_FEE; // 5%
    state.total_debt_amount = 0;
    state.total_stake_amount = 0;
    
    msg!("Aerospacer Protocol initialized successfully");
    msg!("Admin: {}", state.admin);
    msg!("Stable Coin: {}", state.stable_coin_addr);
    msg!("Oracle Helper: {}", state.oracle_helper_addr);
    msg!("Fee Distributor: {}", state.fee_distributor_addr);
    msg!("Minimum Collateral Ratio: {}%", state.minimum_collateral_ratio);
    msg!("Protocol Fee: {}%", state.protocol_fee);
    
    Ok(())
} 