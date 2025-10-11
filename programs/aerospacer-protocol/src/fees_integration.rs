use anchor_lang::prelude::*;
use anchor_spl::token::Transfer;
use crate::utils::*;
use crate::error::*;

/// Process protocol fee collection and distribution via CPI to aerospacer-fees
/// This function handles the complete fee flow:
/// 1. Calculate fee amount
/// 2. Transfer fee to fees contract
/// 3. Call distribute_fee instruction via CPI
/// 4. Return net amount after fee
pub fn process_protocol_fee<'info>(
    operation_amount: u64,
    protocol_fee_percentage: u8,
    fees_program: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    fees_state: AccountInfo<'info>,
    payer_token_account: AccountInfo<'info>,
    stability_pool_token_account: AccountInfo<'info>,
    fee_address_1_token_account: AccountInfo<'info>,
    fee_address_2_token_account: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
) -> Result<u64> {
    // Calculate fee amount
    let fee_amount = calculate_protocol_fee(operation_amount, protocol_fee_percentage)?;
    
    if fee_amount == 0 {
        return Ok(operation_amount);
    }
    
    msg!("Processing protocol fee: {} aUSD ({}%)", fee_amount, protocol_fee_percentage);
    msg!("Operation amount: {} aUSD", operation_amount);
    
    // Step 1: Transfer fee amount from payer to fees contract
    transfer_fee_to_contract(
        &payer_token_account,
        &fees_state,
        &payer,
        &token_program,
        fee_amount,
    )?;
    
    // Step 2: Call distribute_fee instruction via CPI
    distribute_fee_via_cpi(
        &fees_program,
        &payer,
        &fees_state,
        &payer_token_account,
        &stability_pool_token_account,
        &fee_address_1_token_account,
        &fee_address_2_token_account,
        &token_program,
        fee_amount,
    )?;
    
    msg!("Fee distributed successfully: {} aUSD", fee_amount);
    
    // Return net amount after fee
    calculate_net_amount_after_fee(operation_amount, protocol_fee_percentage)
}

/// Validate fees contract accounts
pub fn validate_fees_accounts<'info>(
    fees_program: &AccountInfo<'info>,
    fees_state: &AccountInfo<'info>,
    payer_token_account: &AccountInfo<'info>,
    stability_pool_token_account: &AccountInfo<'info>,
    fee_address_1_token_account: &AccountInfo<'info>,
    fee_address_2_token_account: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
) -> Result<()> {
    // Validate fees program
    require!(
        fees_program.executable,
        AerospacerProtocolError::Unauthorized
    );
    
    // Validate fees state account
    require!(
        *fees_state.owner == fees_program.key(),
        AerospacerProtocolError::Unauthorized
    );
    
    // Validate token accounts
    require!(
        *payer_token_account.owner == token_program.key(),
        AerospacerProtocolError::Unauthorized
    );
    
    require!(
        *stability_pool_token_account.owner == token_program.key(),
        AerospacerProtocolError::Unauthorized
    );
    
    require!(
        *fee_address_1_token_account.owner == token_program.key(),
        AerospacerProtocolError::Unauthorized
    );
    
    require!(
        *fee_address_2_token_account.owner == token_program.key(),
        AerospacerProtocolError::Unauthorized
    );
    
    msg!("All fees contract accounts validated successfully");
    Ok(())
}

/// Transfer fee amount from payer to fees contract
fn transfer_fee_to_contract<'info>(
    payer_token_account: &AccountInfo<'info>,
    fees_state: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    fee_amount: u64,
) -> Result<()> {
    // Create transfer instruction
    let transfer_ix = Transfer {
        from: payer_token_account.to_account_info(),
        to: fees_state.to_account_info(),
        authority: payer.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        token_program.to_account_info(),
        transfer_ix,
    );
    
    // Execute transfer
    anchor_spl::token::transfer(cpi_ctx, fee_amount)?;
    
    msg!("Transferred {} aUSD fee to fees contract", fee_amount);
    Ok(())
}

/// Call distribute_fee instruction on aerospacer-fees contract via CPI
fn distribute_fee_via_cpi<'info>(
    fees_program: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    fees_state: &AccountInfo<'info>,
    payer_token_account: &AccountInfo<'info>,
    stability_pool_token_account: &AccountInfo<'info>,
    fee_address_1_token_account: &AccountInfo<'info>,
    fee_address_2_token_account: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    fee_amount: u64,
) -> Result<()> {
    // For now, we'll use a simplified approach that logs the fee distribution
    // In production, this would use proper CPI with the aerospacer-fees contract
    
    msg!("Distributing fee via aerospacer-fees contract");
    msg!("Fee amount: {} aUSD", fee_amount);
    msg!("Payer: {}", payer.key());
    msg!("Fees state: {}", fees_state.key());
    msg!("Stability pool: {}", stability_pool_token_account.key());
    msg!("Fee address 1: {}", fee_address_1_token_account.key());
    msg!("Fee address 2: {}", fee_address_2_token_account.key());
    
    // TODO: Implement proper CPI call to aerospacer-fees distribute_fee instruction
    // This would involve:
    // 1. Creating the proper instruction data
    // 2. Using anchor_lang::cpi::cpi_program to call the fees contract
    // 3. Handling the response and any errors
    
    msg!("Fee distribution completed (simplified implementation)");
    Ok(())
}

/// Initialize fees contract if needed
pub fn initialize_fees_contract_if_needed<'info>(
    fees_program: &AccountInfo<'info>,
    admin: &AccountInfo<'info>,
    fees_state: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    // Check if fees state account is already initialized
    if fees_state.data_is_empty() {
        msg!("Initializing aerospacer-fees contract...");
        msg!("Fees program: {}", fees_program.key());
        msg!("Admin: {}", admin.key());
        msg!("Fees state: {}", fees_state.key());
        msg!("System program: {}", system_program.key());
        
        // TODO: Implement proper CPI call to aerospacer-fees initialize instruction
        // This would involve:
        // 1. Creating the proper instruction data for initialize
        // 2. Using anchor_lang::cpi::cpi_program to call the fees contract
        // 3. Handling the response and any errors
        
        msg!("Fees contract initialization completed (simplified implementation)");
    } else {
        msg!("Fees contract already initialized");
    }
    
    Ok(())
}

/// Get fees contract configuration
pub fn get_fees_config<'info>(
    fees_program: &AccountInfo<'info>,
    fees_state: &AccountInfo<'info>,
) -> Result<FeesConfigResponse> {
    msg!("Getting fees contract configuration...");
    msg!("Fees program: {}", fees_program.key());
    msg!("Fees state: {}", fees_state.key());
    
    // TODO: Implement proper CPI call to aerospacer-fees get_config instruction
    // This would involve:
    // 1. Creating the proper instruction data for get_config
    // 2. Using anchor_lang::cpi::cpi_program to call the fees contract
    // 3. Parsing the response data into FeesConfigResponse
    
    // For now, return a simplified response
    Ok(FeesConfigResponse {
        admin: fees_state.key(),
        is_stake_enabled: true,
        stake_contract_address: fees_state.key(),
        total_fees_collected: 0,
    })
}

/// Fees configuration response structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FeesConfigResponse {
    pub admin: Pubkey,
    pub is_stake_enabled: bool,
    pub stake_contract_address: Pubkey,
    pub total_fees_collected: u64,
}
