use anchor_lang::prelude::*;
use crate::state::OracleStateAccount;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeParams {
    pub oracle_address: Pubkey,
}

#[derive(Accounts)]
#[instruction(params: InitializeParams)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + OracleStateAccount::LEN)]
    pub state: Account<'info, OracleStateAccount>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    state.admin = ctx.accounts.admin.key();
    state.oracle_address = params.oracle_address;
    state.collateral_data = Vec::new(); // Initialize empty vector
    
    msg!("Aerospacer Oracle initialized successfully");
    msg!("Admin: {}", state.admin);
    msg!("Oracle Address: {}", state.oracle_address);
    
    Ok(())
}


