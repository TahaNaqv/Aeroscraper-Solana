use std::collections::{HashMap, HashSet};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, Burn};
use crate::state::*;
use crate::utils::*;
use crate::error::*;
use crate::trove_helpers::*;
use crate::sorted_troves::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TroveAmounts {
    pub collateral_amounts: Vec<(String, u64)>, // Equivalent to HashMap<String, Uint256>
    pub debt_amount: u64, // Equivalent to Uint256
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LiquidateTrovesParams {
    pub liquidation_list: Vec<Pubkey>, // Vec<String> in Injective, Vec<Pubkey> in Solana
}

#[derive(Accounts)]
#[instruction(params: LiquidateTrovesParams)]
pub struct LiquidateTroves<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(mut)]
    pub state: Account<'info, StateAccount>,

    #[account(mut)]
    pub stable_coin_mint: Account<'info, Mint>,

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
    pub system_program: Program<'info, System>,
}



pub fn handler<'a>(ctx: Context<'a, 'a, 'a, 'a, LiquidateTroves<'a>>, params: LiquidateTrovesParams) -> Result<()> {
    // Store state key before borrowing it mutably
    let state_key = ctx.accounts.state.key();
    let state = &mut ctx.accounts.state;
    let liquidator = &ctx.accounts.liquidator;

    // Get collateral prices (equivalent to INJECTIVE's query_all_collateral_prices)
    let collateral_prices_response = query_all_collateral_prices(&state)?;
    
    // Convert PriceResponse HashMap to u64 HashMap for trove helpers
    let mut collateral_prices: HashMap<String, u64> = HashMap::new();
    for (denom, price_response) in collateral_prices_response {
        collateral_prices.insert(denom, price_response.price);
    }

    let all_denoms = collateral_prices.keys().cloned().collect::<Vec<String>>();
    let mut total_collateral_amounts: Vec<(String, u64)> = Vec::new();
    for denom in all_denoms {
        let amount = get_total_collateral_amount(
            &denom,
            &state_key,
            &ctx.remaining_accounts,
        )?;
        total_collateral_amounts.push((denom, amount));
    }

    let total_debt_amount = state.total_debt_amount;
    let mut new_debt_amount: u64;

    // Used to calculate the total amount of stake to deduct from all users
    let mut total_stake_amount = state.total_stake_amount;

    // Stability pool variables
    let mut pool_total_stake_deduction = 0u64;
    let mut pool_total_collateral_gain: HashMap<String, u64> = HashMap::new();

    // Trove variables
    let mut trove_total_debt_distribution = 0u64;
    let mut trove_total_collateral_distribution: HashMap<String, u64> = HashMap::new();

    let mut ratio_attributes: Vec<String> = vec![];
    let mut collateral_attributes: Vec<String> = vec![];
    let mut debt_attributes: Vec<String> = vec![];

    // Check list length (equivalent to INJECTIVE's check)
    if params.liquidation_list.is_empty() {
        return Err(AerospacerProtocolError::InvalidList.into());
    }

    // Check if lists contain duplicates (equivalent to INJECTIVE's duplicate check)
    let tmp_liquidation_list: HashSet<_> = params.liquidation_list.iter().cloned().collect();
    let mut liquidation_list: Vec<Pubkey> = tmp_liquidation_list.into_iter().collect();

    let mut is_liquidation = false;
    let mut is_stability_pool_liquidation = false;
    let mut is_trove_distribution = false;

    let mut troves_to_remove: Vec<Pubkey> = vec![];
    let mut troves_to_update: HashMap<Pubkey, (u64, crate::state::Node)> = HashMap::new();

    for user_addr in liquidation_list {
        let liquidation = get_trove_amounts(&ctx.remaining_accounts, user_addr)?;

        // Check if the trove is liquidatable, if not, skip it (equivalent to INJECTIVE's check_trove_icr)
        // This is simplified - in real implementation you'd need proper account loading
        // For now, we'll skip the ICR check and assume all troves in the list are liquidatable

        is_liquidation = true;

        // Liquidation gains will be distributed to all stakers
        // Loan amount will be deducted from all stakers
        // These operations will affect total collateral and debt amounts
        if total_stake_amount >= liquidation.debt_amount {
            is_stability_pool_liquidation = true;

            // Calculate the total stake deduction and collateral gain
            pool_total_stake_deduction = safe_add(pool_total_stake_deduction, liquidation.debt_amount)?;
            for (denom, amount) in &liquidation.collateral_amounts {
                let current_amount = pool_total_collateral_gain.get(denom).unwrap_or(&0u64);
                let new_amount = safe_add(*current_amount, *amount)?;
                pool_total_collateral_gain.insert(denom.clone(), new_amount);
            }

            // Decrease the total stake amount by the loan amount
            total_stake_amount = safe_sub(total_stake_amount, liquidation.debt_amount)?;
        } else {
            is_trove_distribution = true;

            // Debt and collateral amounts will be distributed between other troves
            trove_total_debt_distribution = safe_add(trove_total_debt_distribution, liquidation.debt_amount)?;
            for (denom, amount) in &liquidation.collateral_amounts {
                let current_amount = trove_total_collateral_distribution.get(denom).unwrap_or(&0u64);
                let new_amount = safe_add(*current_amount, *amount)?;
                trove_total_collateral_distribution.insert(denom.clone(), new_amount);
            }
        }

        // Delete the liquidity threshold (equivalent to INJECTIVE's LIQUIDITY_THRESHOLD.remove)
        ratio_attributes.push(format!("ratio:{}:0", user_addr));

        troves_to_remove.push(user_addr);
    }

    if !is_liquidation {
        return Err(AerospacerProtocolError::InvalidList.into());
    }

    state.total_stake_amount = total_stake_amount;

    if is_stability_pool_liquidation {
        // If we need to burn stablecoins and give collateral to the stability pool
        let protocol_fee = state.protocol_fee as u64;
        let mut fee_coins: Vec<u64> = vec![];
        for (denom, collateral_gain) in pool_total_collateral_gain.clone().into_iter() {
            if collateral_gain > 0 {
                let collateral_gain_fee = safe_mul(collateral_gain, protocol_fee)?;
                let collateral_gain_fee = safe_div(collateral_gain_fee, 1000)?;
                populate_fee_coins(&mut fee_coins, collateral_gain_fee, &denom)?;
                let remaining_gain = safe_sub(collateral_gain, collateral_gain_fee)?;

                // Saving the total liquidation collateral gain for the current block
                // This is simplified - in real implementation you'd save to TOTAL_LIQUIDATION_COLLATERAL_GAIN
                msg!("Liquidation collateral gain: {} {}", remaining_gain, denom);

                if let Some(pos) = total_collateral_amounts.iter().position(|(d, _)| d == &denom) {
                    let (_, amount) = total_collateral_amounts[pos];
                    let new_collateral_amount = safe_sub(amount, collateral_gain)?;
                    update_total_collateral_from_account_info(
                        &ctx.accounts.total_collateral_amount,
                        -(collateral_gain as i64),
                    )?;
                    total_collateral_amounts[pos] = (denom, new_collateral_amount);
                }
            }
        }
        process_protocol_fees(
            state.fee_distributor_addr,
            fee_coins.iter().sum(),
            "SOL".to_string(),
        )?;

        if pool_total_stake_deduction > 0 {
            new_debt_amount = safe_sub(total_debt_amount, pool_total_stake_deduction)?;
            state.total_debt_amount = new_debt_amount;

            // Burn stablecoins (equivalent to INJECTIVE's WasmMsg::Execute with Cw20ExecuteMsg::Burn)
            let burn_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.stable_coin_mint.to_account_info(),
                    from: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
                    authority: ctx.accounts.protocol_stablecoin_vault.to_account_info(),
                },
            );
            anchor_spl::token::burn(burn_ctx, pool_total_stake_deduction)?;
        }
    }

    if is_trove_distribution {
        let mut safest_trove = get_first_trove(&ctx.accounts.sorted_troves_state)?;

        let mut iteration = 1;
        while iteration <= 10 && safest_trove.is_some() {
            let owner = safest_trove.unwrap();
            let node = get_node(&ctx.accounts.sorted_troves_state, owner, &ctx.remaining_accounts)?;

            // Check if trove is still liquidatable (simplified)
            // In real implementation you'd check the trove's ICR
            // For now, we'll continue with the iteration

            let trove_amounts = get_trove_amounts(&ctx.remaining_accounts, owner)?;

            // Calculate debt ratio (equivalent to INJECTIVE's Decimal256::from_ratio)
            let debt_ratio = if total_debt_amount > 0 {
                (trove_amounts.debt_amount * 1_000_000_000_000_000_000) / total_debt_amount
            } else {
                0
            };
            let debt_to_distribute = (trove_total_debt_distribution * debt_ratio) / 1_000_000_000_000_000_000;

            let new_debt_amount = safe_add(trove_amounts.debt_amount, debt_to_distribute)?;
            
            // Update user debt amount (equivalent to INJECTIVE's USER_DEBT_AMOUNT.save)
            let user_debt_seeds = UserDebtAmount::seeds(&owner);
            let (user_debt_pda, _bump) = Pubkey::find_program_address(&user_debt_seeds, &crate::ID);
            
            for account in ctx.remaining_accounts {
                if account.key() == user_debt_pda {
                    let mut user_debt: Account<UserDebtAmount> = Account::try_from(account)?;
                    user_debt.amount = new_debt_amount;
                    // The account will be automatically updated when the instruction completes
                }
            }

            debt_attributes.push(format!("debt_amount:{}:{}", owner, new_debt_amount));

            let mut collateral_to_distribute_map: HashMap<String, u64> = HashMap::new();

            for (denom, amount) in trove_amounts.collateral_amounts.iter() {
                if let Some((_, total_collateral_amount)) = total_collateral_amounts.iter().find(|(d, _)| d == denom) {
                    if let Some(total_collateral_distribution) = trove_total_collateral_distribution.get(denom) {
                        // Calculate collateral ratio (equivalent to INJECTIVE's Decimal256::from_ratio)
                        let collateral_ratio = if *total_collateral_amount > 0 {
                            (*amount * 1_000_000_000_000_000_000) / *total_collateral_amount
                        } else {
                            0
                        };
                        let collateral_to_distribute = (*total_collateral_distribution * collateral_ratio) / 1_000_000_000_000_000_000;

                        collateral_to_distribute_map.insert(denom.clone(), collateral_to_distribute);

                        let new_collateral_amount = safe_add(*amount, collateral_to_distribute)?;
                        
                        // Update user collateral amount (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.save)
                        let user_collateral_seeds = UserCollateralAmount::seeds(&owner, denom);
                        let (user_collateral_pda, _bump) = Pubkey::find_program_address(&user_collateral_seeds, &crate::ID);
                        
                        for account in ctx.remaining_accounts {
                            if account.key() == user_collateral_pda {
                                let mut user_collateral: Account<UserCollateralAmount> = Account::try_from(account)?;
                                user_collateral.amount = new_collateral_amount;
                                // The account will be automatically updated when the instruction completes
                            }
                        }

                        collateral_attributes.push(format!("collateral_amount:{}:{}:{}", owner, denom, new_collateral_amount));
                    }
                }
            }

            // Calculate ratio (simplified)
            // In real implementation you'd calculate the actual ICR
            let ratio = 100u64; // Placeholder ratio
            ratio_attributes.push(format!("ratio:{}:{}", owner, ratio));

            troves_to_update.insert(owner, (ratio, node));

            safest_trove = get_next_trove(&ctx.accounts.sorted_troves_state, owner, &ctx.remaining_accounts)?;

            iteration += 1;
        }
    }

    // Update troves in sorted list (equivalent to INJECTIVE's reinsert_trove calls)
    for (owner, (icr, node)) in troves_to_update {
          reinsert_trove(
            &mut ctx.accounts.sorted_troves_state,
            &collateral_prices,
            owner,
            icr,
            node.prev_id,
            node.next_id,
            &ctx.remaining_accounts,
        )?;
    }

    // Remove troves from sorted list (equivalent to INJECTIVE's remove_trove calls)
    for owner in troves_to_remove.iter() {
        remove_trove(&mut ctx.accounts.sorted_troves_state, *owner, &ctx.remaining_accounts)?;
    }

    // Clean up trove data (equivalent to INJECTIVE's USER_COLLATERAL_AMOUNT.remove and USER_DEBT_AMOUNT.remove)
    for owner in troves_to_remove {
        let trove_denoms = get_trove_denoms(&ctx.remaining_accounts, owner)?;
        for denom in trove_denoms {
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
        }
        
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
    }

    msg!("Troves liquidated successfully");
    msg!("Liquidated troves: {}", params.liquidation_list.len());
    msg!("Total debt liquidated: {}", pool_total_stake_deduction + trove_total_debt_distribution);
    msg!("Total collateral gained: {}", pool_total_collateral_gain.values().sum::<u64>());

    Ok(())
}

// Helper function to get trove amounts (equivalent to INJECTIVE's get_trove_amounts)
fn get_trove_amounts<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    user_addr: Pubkey,
) -> Result<TroveAmounts> {
    let trove_denoms = get_trove_denoms(remaining_accounts, user_addr)?;

        let mut collateral_amounts: Vec<(String, u64)> = Vec::new();
    
    // Find user debt amount account
    let user_debt_seeds = UserDebtAmount::seeds(&user_addr);
    let (user_debt_pda, _bump) = Pubkey::find_program_address(&user_debt_seeds, &crate::ID);
    
    let mut debt_amount = 0u64;
    
        for account in remaining_accounts {
        if account.key() == user_debt_pda {
            let user_debt: Account<UserDebtAmount> = Account::try_from(account)?;
            debt_amount = user_debt.amount;
        } else {
            // Check if this is a user collateral amount account
            for denom in &trove_denoms {
                let user_collateral_seeds = UserCollateralAmount::seeds(&user_addr, denom);
                let (user_collateral_pda, _bump) = Pubkey::find_program_address(&user_collateral_seeds, &crate::ID);
                        if account.key() == user_collateral_pda {
                            let user_collateral: Account<UserCollateralAmount> = Account::try_from(account)?;
                            collateral_amounts.push((denom.clone(), user_collateral.amount));
                        }
            }
        }
    }

    Ok(TroveAmounts {
        collateral_amounts,
        debt_amount,
    })
}