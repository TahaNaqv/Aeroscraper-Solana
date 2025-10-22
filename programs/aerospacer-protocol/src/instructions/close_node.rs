use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use crate::state::{Node, StateAccount};
use crate::error::AerospacerProtocolError;

#[derive(Accounts)]
#[instruction(user_pubkey: Pubkey)]
pub struct CloseNode<'info> {
    /// CHECK: Using UncheckedAccount to bypass deserialization for corrupted discriminators
    /// Seeds and bump are validated, then account is closed manually
    #[account(
        mut,
        seeds = [b"node", user_pubkey.as_ref()],
        bump,
    )]
    pub node: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"state"],
        bump,
        constraint = state.admin == authority.key() @ AerospacerProtocolError::Unauthorized
    )]
    pub state: Box<Account<'info, StateAccount>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CloseNode>, user_pubkey: Pubkey) -> Result<()> {
    msg!("⚠️  ADMIN: Closing Node account for emergency recovery");
    msg!("  User: {}", user_pubkey);
    msg!("  Node PDA: {}", ctx.accounts.node.key());
    
    let node_data = ctx.accounts.node.try_borrow_data()?;
    msg!("  Account data length: {} bytes", node_data.len());
    let node_lamports = ctx.accounts.node.lamports();
    msg!("  Account lamports: {} lamports", node_lamports);
    
    if node_data.len() >= 8 {
        let discriminator = &node_data[0..8];
        msg!("  Discriminator (first 8 bytes): {:?}", discriminator);
        
        let expected_discriminator: [u8; 8] = [198, 82, 111, 206, 177, 93, 160, 219];
        if discriminator == expected_discriminator {
            msg!("  ✓ Valid Node discriminator detected");
        } else {
            msg!("  ⚠️  CORRUPTED: Invalid discriminator (this is why we're closing it)");
        }
    } else {
        msg!("  ⚠️  Account too small or empty");
    }
    drop(node_data);
    
    msg!("📤 Transferring {} lamports to authority...", node_lamports);
    
    **ctx.accounts.node.to_account_info().try_borrow_mut_lamports()? = 0;
    **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += node_lamports;
    
    ctx.accounts.node.to_account_info().realloc(0, false)?;
    ctx.accounts.node.to_account_info().assign(&anchor_lang::system_program::ID);
    
    msg!("✅ Node account closed - user can open fresh trove");
    msg!("⚠️  WARNING: This Node is removed from sorted list!");
    msg!("   User will need to call openTrove to create a new Node");
    
    Ok(())
}
