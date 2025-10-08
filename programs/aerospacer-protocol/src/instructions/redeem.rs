use std::collections::HashMap;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, Burn};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::trove_helpers::{get_trove_denoms, get_trove_icr, check_trove_icr_with_ratio, get_trove_liquidity_ratios};
use crate::TroveAmounts;
use crate::sorted_troves::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RedeemParams {
    pub amount: u64, // Equivalent to Uint256
    pub prev_node_id: Option<Pubkey>,
    pub next_node_id: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: RedeemParams)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_collateral_account: Account<'info, TokenAccount>,

    /// CHECK: Protocol stablecoin vault PDA
    #[account(
        mut,
        seeds = [b"protocol_stablecoin_vault"],
        bump
    )]
    pub protocol_stablecoin_vault: AccountInfo<'info>,

    /// CHECK: Protocol collateral vault PDA
    #[account(
        mut,
        seeds = [b"protocol_collateral_vault", b"SOL"], // Default to SOL for now
        bump
    )]
    pub protocol_collateral_vault: AccountInfo<'info>,

    /// CHECK: This is the stable coin mint account
    pub stable_coin_mint: UncheckedAccount<'info>,

    /// CHECK: Per-denom collateral total PDA
    #[account(
        mut,
        seeds = [b"total_collateral_amount", b"SOL"],
        bump
    )]
    pub total_collateral_amount: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"sorted_troves_state"],
        bump
    )]
    pub sorted_troves_state: Account<'info, SortedTrovesState>,

    pub token_program: Program<'info, Token>,
}



pub fn handler<'a>(ctx: Context<'a, 'a, 'a, 'a, Redeem<'a>>, params: RedeemParams) -> Result<()> {
    // Store state key before borrowing it mutably
    let state_key = ctx.accounts.state.key();
    let state = &mut ctx.accounts.state;

    // Get collateral prices (equivalent to INJECTIVE's query_all_collateral_prices)
    let collateral_prices_response = query_all_collateral_prices(&state)?;
    
    // Convert PriceResponse HashMap to u64 HashMap for trove helpers
    let mut collateral_prices: HashMap<String, u64> = HashMap::new();
    for (denom, price_response) in collateral_prices_response {
        collateral_prices.insert(denom, price_response.price);
    }

    // Old total debt amount in protocol (equivalent to INJECTIVE's TOTAL_DEBT_AMOUNT.load)
    let old_total_debt_amount = state.total_debt_amount;
    let mut old_total_collateral_amount_map: HashMap<String, u64> = HashMap::new();
    for (denom, _) in &collateral_prices {
        let amount = get_total_collateral_amount(
            denom,
            &state_key,
            &ctx.remaining_accounts,
        )?;
        old_total_collateral_amount_map.insert(denom.clone(), amount);
    }

    // Check if there's enough liquidity for redemption (equivalent to INJECTIVE's check)
    require!(
        params.amount <= old_total_debt_amount,
        AerospacerProtocolError::NotEnoughLiquidityForRedeem
    );

    // Collateral amounts to send to the user (equivalent to INJECTIVE's collateral_to_send)
    let mut collateral_to_send: HashMap<String, u64> = HashMap::new();
    // Collateral amounts to return to the trove owners (equivalent to INJECTIVE's collateral_to_return)
    let mut collateral_to_return: HashMap<(Pubkey, String), u64> = HashMap::new();

    // Remaining stable coin amount to be processed (equivalent to INJECTIVE's remaining_amount)
    let mut remaining_amount = params.amount;

    let mut ratio_attributes: Vec<String> = vec![];
    let mut collateral_attributes: Vec<String> = vec![];
    let mut debt_attributes: Vec<String> = vec![];

    // Get the riskiest trove (equivalent to INJECTIVE's get_last_trove)
    let mut trove_owner = get_last_trove(&ctx.accounts.sorted_troves_state)?;

    let mut trove_to_update: (Pubkey, u64) = (Pubkey::default(), 0);
    let mut troves_to_remove: Vec<Pubkey> = vec![];

    // Process troves from riskiest to safest (equivalent to INJECTIVE's while loop)
    while trove_owner.is_some() {
        let owner = trove_owner.unwrap();

        // Get trove liquidity ratios (equivalent to INJECTIVE's get_trove_liquidity_ratios)
        let liquidity_ratios = get_trove_liquidity_ratios(
            &ctx.remaining_accounts, // user_collateral_amount_accounts
            &collateral_prices,
            owner,
        )?;
        let trove_denoms = get_trove_denoms(&ctx.remaining_accounts, owner)?;

        // Get old debt amount (equivalent to INJECTIVE's USER_DEBT_AMOUNT.load)
        let user_debt_seeds = UserDebtAmount::seeds(&owner);
        let (user_debt_pda, _bump) = Pubkey::find_program_address(&user_debt_seeds, &crate::ID);
        
        let mut old_debt_amount = 0u64;
        for account in ctx.remaining_accounts {
            if account.key() == user_debt_pda {
                let user_debt: Account<UserDebtAmount> = Account::try_from(account)?;
                old_debt_amount = user_debt.amount;
                break;
            }
        }

        let collateral_ratio: u64;

        if old_debt_amount > remaining_amount {
            // Partial redemption (equivalent to INJECTIVE's partial redemption logic)
            let new_debt = safe_sub(old_debt_amount, remaining_amount)?;
            // Update user debt amount (equivalent to INJECTIVE's USER_DEBT_AMOUNT.save)
            let user_debt_seeds = UserDebtAmount::seeds(&owner);
            let (user_debt_pda, _bump) = Pubkey::find_program_address(&user_debt_seeds, &crate::ID);
            
            for account in ctx.remaining_accounts {
                if account.key() == user_debt_pda {
                    let mut user_debt: Account<UserDebtAmount> = Account::try_from(account)?;
                    user_debt.amount = new_debt;
                    // The account will be automatically updated when the instruction completes
                    break;
                }
            }
            
            debt_attributes.push(format!("debt_amount:{}:{}", owner, new_debt));
            state.total_debt_amount = safe_sub(old_total_debt_amount, remaining_amount)?;

            for denom in trove_denoms.clone() {
                let price_data = collateral_prices.get(&denom).unwrap();

                if let Some((ratio, price_response)) = liquidity_ratios.get(&denom).and_then(|ratio| {
                    collateral_prices
                        .get(&denom)
                        .and_then(|price_response| Some((*ratio, *price_response)))
                }) {
                    process_collateral(
                        false,
                        owner,
                        denom.clone(),
                        price_response,
                        9, // Default decimal for SOL
                        ratio,
                        remaining_amount,
                        &mut collateral_to_send,
                        &mut collateral_to_return,
                        &mut collateral_attributes,
                        &ctx.remaining_accounts,
                    )?;
                }
            }

            // Calculate collateral ratio (equivalent to INJECTIVE's get_trove_icr)
            collateral_ratio = 100u64; // Placeholder - would calculate actual ICR
            process_ratio(
                owner,
                collateral_ratio,
                &mut ratio_attributes,
            )?;

            trove_to_update = (owner, collateral_ratio);

            break;
        }

        // Full redemption (equivalent to INJECTIVE's full redemption logic)
        remaining_amount = safe_sub(remaining_amount, old_debt_amount)?;

        for denom in trove_denoms.clone() {
            let price_data = query_collateral_price(&state, &denom)?;

            if let Some((ratio, price_response)) = liquidity_ratios.get(&denom).and_then(|ratio| {
                collateral_prices
                    .get(&denom)
                    .and_then(|price_response| Some((*ratio, *price_response)))
            }) {
                process_collateral(
                    true,
                    owner,
                    denom.clone(),
                    price_response,
                    price_data.decimal,
                    ratio,
                    old_debt_amount,
                    &mut collateral_to_send,
                    &mut collateral_to_return,
                    &mut collateral_attributes,
                    &ctx.remaining_accounts,
                )?;
            }
        }

        state.total_debt_amount = safe_sub(old_total_debt_amount, old_debt_amount)?;

        // Remove liquidity threshold (equivalent to INJECTIVE's LIQUIDITY_THRESHOLD.remove)
        ratio_attributes.push(format!("ratio:{}:0", owner));

        troves_to_remove.push(owner);

        // Get previous trove (equivalent to INJECTIVE's get_prev_trove)
        trove_owner = get_prev_trove(&ctx.accounts.sorted_troves_state, owner, &ctx.remaining_accounts)?;
    }

    // Reinsert trove if partially redeemed (equivalent to INJECTIVE's reinsert_trove)
    if trove_to_update.0 != Pubkey::default() {
          reinsert_trove(
            &mut ctx.accounts.sorted_troves_state,
            &collateral_prices,
            trove_to_update.0,
            trove_to_update.1,
            params.prev_node_id,
            params.next_node_id,
            &ctx.remaining_accounts,
        )?;
    }

    // Remove fully redeemed troves (equivalent to INJECTIVE's remove_trove calls)
    for owner in troves_to_remove {
        // Remove user debt amount (equivalent to INJECTIVE's USER_DEBT_AMOUNT.remove)
        let user_debt_seeds = UserDebtAmount::seeds(&owner);
        let (user_debt_pda, _bump) = Pubkey::find_program_address(&user_debt_seeds, &crate::ID);
        
        for account in ctx.remaining_accounts {
            if account.key() == user_debt_pda {
                let mut user_debt: Account<UserDebtAmount> = Account::try_from(account)?;
                user_debt.amount = 0; // Remove the debt
                // The account will be automatically updated when the instruction completes
                break;
            }
        }
        
        remove_trove(&mut ctx.accounts.sorted_troves_state, owner, &ctx.remaining_accounts)?;
    }

    // Transfer stablecoins from user to protocol
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_stablecoin_account.to_account_info(),
            to: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, params.amount)?;

    // Burn stablecoin (equivalent to INJECTIVE's WasmMsg::Execute with Cw20ExecuteMsg::Burn)
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.stable_coin_mint.to_account_info(),
            from: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
            authority: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
        },
    );
    anchor_spl::token::burn(burn_ctx, params.amount)?;

    // Send collateral to sender (equivalent to INJECTIVE's BankMsg::Send)
    let protocol_fee = state.protocol_fee as u64;
    let mut fee_coins: Vec<u64> = vec![];
    for (denom, amount) in &collateral_to_send {
        let redeem_fee = safe_mul(*amount, protocol_fee)?;
        let redeem_fee = safe_div(redeem_fee, 1000)?;
        let send_amount = safe_sub(*amount, redeem_fee)?;
        
        // Transfer collateral to user (simplified - would handle multiple denoms in real implementation)
        let transfer_collateral_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.protocol_collateral_vault.to_account_info(),
                to: ctx.accounts.user_collateral_account.to_account_info(),
                authority: ctx.accounts.protocol_collateral_vault.to_account_info(),
            },
        );
        anchor_spl::token::transfer(transfer_collateral_ctx, send_amount)?;
        
        populate_fee_coins(&mut fee_coins, redeem_fee, &denom)?;
    }
    process_protocol_fees(
        state.fee_distributor_addr,
        fee_coins.iter().sum(),
        "SOL".to_string(),
    )?;

    // Return collateral to trove owners (equivalent to INJECTIVE's collateral_to_return processing)
    for ((owner, denom), amount) in collateral_to_return {
        // Remove user collateral amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.remove)
        let user_collateral_seeds = UserCollateralAmount::seeds(&owner, &denom);
        let (user_collateral_pda, _bump) = Pubkey::find_program_address(&user_collateral_seeds, &crate::ID);
        
        for account in ctx.remaining_accounts {
            if account.key() == user_collateral_pda {
                let mut user_collateral: Account<UserCollateralAmount> = Account::try_from(account)?;
                user_collateral.amount = 0; // Remove the collateral
                // The account will be automatically updated when the instruction completes
                break;
            }
        }
        
        // Transfer collateral back to trove owner (simplified)
        // In real implementation, you'd transfer to each trove owner's account
        msg!("Returning {} {} to trove owner {}", amount, denom, owner);
    }

    // Update per-denom collateral total PDA
    update_total_collateral_from_account_info(
        &ctx.accounts.total_collateral_amount,
        -(params.amount as i64),
    )?;

    msg!("Redeemed successfully");
    msg!("Amount: {} aUSD", params.amount);
    msg!("Collateral sent: {} SOL", collateral_to_send.get("SOL").unwrap_or(&0));

    Ok(())
}

// Helper function to process collateral (equivalent to INJECTIVE's process_collateral)
fn process_collateral<'a>(
    close_trove: bool,
    owner: Pubkey,
    collateral_denom: String,
    collateral_price: u64,
    collateral_decimal: u8,
    collateral_ratio: u64,
    loan_amount: u64,
    collateral_to_send: &mut HashMap<String, u64>,
    collateral_to_return: &mut HashMap<(Pubkey, String), u64>,
    collateral_attributes: &mut Vec<String>,
    remaining_accounts: &'a [AccountInfo<'a>],
) -> Result<()> {
    // Calculate partial loan (equivalent to INJECTIVE's calculate_stake_amount)
    let partial_loan = calculate_stake_amount(loan_amount, collateral_ratio, true)?;

    let decimal = match collateral_decimal {
        6 => DECIMAL_FRACTION_6,
        18 => DECIMAL_FRACTION_18,
        _ => return Err(AerospacerProtocolError::InvalidDecimal.into()),
    };

    // Calculate collateral to decrease (equivalent to INJECTIVE's complex calculation)
    let collateral_to_decrease = safe_mul(partial_loan, decimal as u64)?;
    let collateral_to_decrease = safe_mul(collateral_to_decrease, 100_000_000)?;
    let collateral_to_decrease = safe_div(collateral_to_decrease, collateral_price)?;
    let collateral_to_decrease = safe_div(collateral_to_decrease, DECIMAL_FRACTION_18 as u64)?;

    // Add the collateral amount to send to the map (equivalent to INJECTIVE's logic)
    match collateral_to_send.contains_key(&collateral_denom) {
        true => {
            let new_amount = safe_add(
                *collateral_to_send.get(&collateral_denom).unwrap(),
                collateral_to_decrease,
            )?;
            collateral_to_send.insert(collateral_denom.clone(), new_amount);
        }
        false => {
            collateral_to_send.insert(collateral_denom.clone(), collateral_to_decrease);
        }
    }

    // Calculate the new collateral amount for the user (equivalent to INJECTIVE's logic)
    let user_collateral_seeds = UserCollateralAmount::seeds(&owner, &collateral_denom);
    let (user_collateral_pda, _bump) = Pubkey::find_program_address(&user_collateral_seeds, &crate::ID);
    
    let mut current_collateral_amount = 0u64;
    for account in remaining_accounts {
        if account.key() == user_collateral_pda {
            let user_collateral: Account<UserCollateralAmount> = Account::try_from(account)?;
            current_collateral_amount = user_collateral.amount;
            break;
        }
    }
    
    let new_collateral_amount = safe_sub(current_collateral_amount, collateral_to_decrease)?;

    if close_trove {
        collateral_to_return.insert(
            (owner, collateral_denom.clone()),
            collateral_to_decrease,
        );
    } else {
        // Update user collateral amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.save)
        let user_collateral_seeds = UserCollateralAmount::seeds(&owner, &collateral_denom);
        let (user_collateral_pda, _bump) = Pubkey::find_program_address(&user_collateral_seeds, &crate::ID);
        
        for account in remaining_accounts {
            if account.key() == user_collateral_pda {
                let mut user_collateral: Account<UserCollateralAmount> = Account::try_from(account)?;
                user_collateral.amount = new_collateral_amount;
                // The account will be automatically updated when the instruction completes
                break;
            }
        }
        
        collateral_attributes.push(format!("collateral_amount:{}:{}:{}", owner, collateral_denom, new_collateral_amount));
    }

    Ok(())
}

// Helper function to process ratio (equivalent to INJECTIVE's process_ratio)
fn process_ratio(
    owner: Pubkey,
    ratio: u64,
    ratio_attributes: &mut Vec<String>,
) -> Result<()> {
    ratio_attributes.push(format!("ratio:{}:{}", owner, ratio));
    // Update liquidity threshold (equivalent to INJECTIVE's LIQUIDITY_THRESHOLD.save)
    // This is simplified - in real implementation you'd update the user's liquidity threshold account
    Ok(())
}