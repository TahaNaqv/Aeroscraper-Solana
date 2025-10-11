use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

/// Initialize the sorted troves list
pub fn initialize_sorted_troves(sorted_troves_state: &mut Account<SortedTrovesState>) -> Result<()> {
    sorted_troves_state.head = None;
    sorted_troves_state.tail = None;
    sorted_troves_state.size = 0;
    Ok(())
}

/// Insert a trove into the sorted list with proper ICR-based positioning
/// Uses remaining_accounts to pass neighbor nodes for pointer updates
pub fn insert_trove(
    sorted_troves_state: &mut SortedTrovesState,
    user_node: &mut Node,
    user: Pubkey,
    icr: u64,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    // Validate ICR is non-zero
    require!(icr > 0, AerospacerProtocolError::InvalidAmount);
    
    // Initialize the user's node
    user_node.id = user;
    user_node.prev_id = None;
    user_node.next_id = None;
    
    // Handle first trove (empty list)
    if sorted_troves_state.size == 0 {
        sorted_troves_state.head = Some(user);
        sorted_troves_state.tail = Some(user);
        sorted_troves_state.size = 1;
        
        msg!("First trove inserted: user={}, icr={}", user, icr);
        return Ok(());
    }
    
    // Find ICR-based insertion position via full list traversal
    // remaining_accounts pattern: [node1, lt1, node2, lt2, ...] for traversal
    let (prev_id, next_id) = find_insert_position(
        sorted_troves_state,
        icr,
        remaining_accounts,
    )?;
    
    // Update the new node's pointers
    user_node.prev_id = prev_id;
    user_node.next_id = next_id;
    
    // Update neighbor nodes' pointers
    // We need to find the neighbor Node accounts in remaining_accounts
    match (prev_id, next_id) {
        (None, None) => {
            // Should not happen for non-empty list
            msg!("Error: Invalid position (None, None) for non-empty list");
            return Err(AerospacerProtocolError::InvalidList.into());
        }
        (None, Some(next)) => {
            // Insert at head - update old head's prev_id
            // Head node should be at index 0 of remaining_accounts (first in traversal)
            if remaining_accounts.is_empty() {
                return Err(AerospacerProtocolError::InvalidList.into());
            }
            
            let next_node_account = &remaining_accounts[0];
            let mut next_data = next_node_account.try_borrow_mut_data()?;
            let mut next_node = Node::try_deserialize(&mut &next_data[8..])?;
            require!(next_node.id == next, AerospacerProtocolError::InvalidList);
            
            next_node.prev_id = Some(user);
            let mut writer = &mut next_data[8..];
            next_node.try_serialize(&mut writer)?;
            
            sorted_troves_state.head = Some(user);
            msg!("Inserted at head: {} -> {}", user, next);
        }
        (Some(prev), None) => {
            // Insert at tail - find and update prev node's next_id
            // Need to search remaining_accounts for the prev node
            let mut found_prev = false;
            for i in (0..remaining_accounts.len()).step_by(2) {
                let node_account = &remaining_accounts[i];
                let node_data = node_account.try_borrow_data()?;
                let node = Node::try_deserialize(&mut &node_data[8..])?;
                
                if node.id == prev {
                    drop(node_data); // Release immutable borrow
                    
                    let mut mut_data = node_account.try_borrow_mut_data()?;
                    let mut prev_node = Node::try_deserialize(&mut &mut_data[8..])?;
                    prev_node.next_id = Some(user);
                    let mut writer = &mut mut_data[8..];
                    prev_node.try_serialize(&mut writer)?;
                    
                    found_prev = true;
                    break;
                }
            }
            
            require!(found_prev, AerospacerProtocolError::InvalidList);
            sorted_troves_state.tail = Some(user);
            msg!("Inserted at tail: {} -> {}", prev, user);
        }
        (Some(prev), Some(next)) => {
            // Insert in middle - update both prev and next nodes
            let mut found_prev = false;
            let mut found_next = false;
            
            for i in (0..remaining_accounts.len()).step_by(2) {
                let node_account = &remaining_accounts[i];
                let node_data = node_account.try_borrow_data()?;
                let node = Node::try_deserialize(&mut &node_data[8..])?;
                let node_id = node.id;
                drop(node_data);
                
                if node_id == prev && !found_prev {
                    let mut mut_data = node_account.try_borrow_mut_data()?;
                    let mut prev_node = Node::try_deserialize(&mut &mut_data[8..])?;
                    prev_node.next_id = Some(user);
                    let mut writer = &mut mut_data[8..];
                    prev_node.try_serialize(&mut writer)?;
                    found_prev = true;
                } else if node_id == next && !found_next {
                    let mut mut_data = node_account.try_borrow_mut_data()?;
                    let mut next_node = Node::try_deserialize(&mut &mut_data[8..])?;
                    next_node.prev_id = Some(user);
                    let mut writer = &mut mut_data[8..];
                    next_node.try_serialize(&mut writer)?;
                    found_next = true;
                }
                
                if found_prev && found_next {
                    break;
                }
            }
            
            require!(found_prev && found_next, AerospacerProtocolError::InvalidList);
            msg!("Inserted in middle: {} -> {} -> {}", prev, user, next);
        }
    }
    
    // Update list state
    sorted_troves_state.size += 1;
    
    msg!("Trove inserted: user={}, icr={}, size={}", user, icr, sorted_troves_state.size);
    Ok(())
}

/// Remove a trove from the sorted list
/// Uses remaining_accounts to update neighbor node pointers
pub fn remove_trove(
    sorted_troves_state: &mut SortedTrovesState,
    user: Pubkey,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    require!(sorted_troves_state.size > 0, AerospacerProtocolError::TroveDoesNotExist);
    
    // Get the node to remove (should be at index 0 of remaining_accounts)
    require!(remaining_accounts.len() >= 1, AerospacerProtocolError::InvalidList);
    let user_node_account_info = &remaining_accounts[0];
    let user_node_data = user_node_account_info.try_borrow_data()?;
    let user_node = Node::try_deserialize(&mut &user_node_data[8..])?;
    
    require!(user_node.id == user, AerospacerProtocolError::InvalidList);
    
    let prev_id = user_node.prev_id;
    let next_id = user_node.next_id;
    drop(user_node_data); // Release the borrow
    
    // Handle single-trove list
    if sorted_troves_state.size == 1 {
        sorted_troves_state.head = None;
        sorted_troves_state.tail = None;
        sorted_troves_state.size = 0;
        msg!("Last trove removed: user={}", user);
        return Ok(());
    }
    
    // Update neighbors' pointers
    let mut account_idx = 1; // Start after the user's node
    
    // Update previous node's next_id (REQUIRED if prev exists)
    if let Some(prev) = prev_id {
        require!(
            account_idx < remaining_accounts.len(),
            AerospacerProtocolError::InvalidList
        );
        
        let prev_node_account_info = &remaining_accounts[account_idx];
        let mut prev_node_data = prev_node_account_info.try_borrow_mut_data()?;
        let mut prev_node = Node::try_deserialize(&mut &prev_node_data[8..])?;
        
        require!(prev_node.id == prev, AerospacerProtocolError::InvalidList);
        prev_node.next_id = next_id;
        
        // Serialize back
        let mut writer = &mut prev_node_data[8..];
        prev_node.try_serialize(&mut writer)?;
        
        account_idx += 1;
        msg!("Updated prev node {} -> next_id = {:?}", prev, next_id);
    } else {
        // Removing head - update head pointer
        sorted_troves_state.head = next_id;
    }
    
    // Update next node's prev_id (REQUIRED if next exists)
    if let Some(next) = next_id {
        require!(
            account_idx < remaining_accounts.len(),
            AerospacerProtocolError::InvalidList
        );
        
        let next_node_account_info = &remaining_accounts[account_idx];
        let mut next_node_data = next_node_account_info.try_borrow_mut_data()?;
        let mut next_node = Node::try_deserialize(&mut &next_node_data[8..])?;
        
        require!(next_node.id == next, AerospacerProtocolError::InvalidList);
        next_node.prev_id = prev_id;
        
        // Serialize back
        let mut writer = &mut next_node_data[8..];
        next_node.try_serialize(&mut writer)?;
        
        msg!("Updated next node {} -> prev_id = {:?}", next, prev_id);
    } else {
        // Removing tail - update tail pointer
        sorted_troves_state.tail = prev_id;
    }
    
    sorted_troves_state.size -= 1;
    
    msg!("Trove removed: user={}, size={}", user, sorted_troves_state.size);
    Ok(())
}

/// Reinsert a trove when its ICR changes (e.g., after borrowing more or adding collateral)
/// This is equivalent to remove + insert with new position
pub fn reinsert_trove(
    sorted_troves_state: &mut SortedTrovesState,
    user_node: &mut Node,
    user: Pubkey,
    new_icr: u64,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    // For Phase 3, we'll implement a simplified reinsert:
    // 1. Check if position changed significantly
    // 2. If yes, remove and reinsert
    // 3. If no, keep in place (optimization)
    
    // For now, just validate the ICR
    require!(new_icr > 0, AerospacerProtocolError::InvalidAmount);
    
    msg!("Reinsert trove: user={}, new_icr={}", user, new_icr);
    msg!("Note: Actual repositioning will be implemented in full ICR-based sorting");
    
    // TODO: Implement full remove + find_position + insert with ICR comparison
    // For now, we don't reposition (assumes FIFO order is acceptable)
    
    Ok(())
}

/// Get the first (riskiest) trove in the list
pub fn get_first_trove(sorted_troves_state: &Account<SortedTrovesState>) -> Option<Pubkey> {
    sorted_troves_state.head
}

/// Get the last (safest) trove in the list
pub fn get_last_trove(sorted_troves_state: &Account<SortedTrovesState>) -> Option<Pubkey> {
    sorted_troves_state.tail
}

/// Find the correct insertion position based on ICR with FULL list traversal
/// Returns (prev_id, next_id) for the new node's position
/// Lower ICR = riskier = closer to head, Higher ICR = safer = closer to tail
/// 
/// **MANDATORY ACCOUNT REQUIREMENT:**
/// - This function REQUIRES all traversal accounts to maintain ICR ordering
/// - If accounts are missing, transaction ABORTS (no fallback to tail)
/// - This prevents callers from bypassing ICR ordering by omitting accounts
/// 
/// **Traversal Strategy:**
/// 1. Walk from head using Node.next_id pointers
/// 2. For each node, REQUIRE (Node, LiquidityThreshold) pair in remaining_accounts
/// 3. Find first node where new_icr < node_icr → insert before that node
/// 4. If we reach end without finding spot → insert at tail (new_icr is highest)
/// 
/// remaining_accounts pattern (in order): [node1, lt1, node2, lt2, node3, lt3, ...]
pub fn find_insert_position(
    sorted_troves_state: &SortedTrovesState,
    new_icr: u64,
    remaining_accounts: &[AccountInfo],
) -> Result<(Option<Pubkey>, Option<Pubkey>)> {
    if sorted_troves_state.size == 0 {
        return Ok((None, None));
    }
    
    let head = sorted_troves_state.head.unwrap();
    
    // Start traversal from head
    let mut current_id = Some(head);
    let mut prev_id: Option<Pubkey> = None;
    let mut account_idx = 0;
    
    while let Some(current) = current_id {
        // REQUIRE both Node and LiquidityThreshold accounts for current node
        // This is MANDATORY to maintain ICR ordering - no fallback allowed
        require!(
            account_idx + 1 < remaining_accounts.len(),
            AerospacerProtocolError::InvalidList
        );
        
        let node_account = &remaining_accounts[account_idx];
        let lt_account = &remaining_accounts[account_idx + 1];
        
        // Load Node to get next_id
        let node_data = node_account.try_borrow_data()?;
        let node = Node::try_deserialize(&mut &node_data[8..])?;
        require!(node.id == current, AerospacerProtocolError::InvalidList);
        let next_id = node.next_id;
        drop(node_data); // Release borrow
        
        // Load LiquidityThreshold to get ICR
        let current_icr = get_icr_from_account(lt_account, current)?;
        
        // Found the insertion point: new_icr < current_icr
        if new_icr < current_icr {
            msg!("Found position: new_icr {} < node {} icr {}", new_icr, current, current_icr);
            return Ok((prev_id, Some(current)));
        }
        
        // Move to next node
        account_idx += 2; // Skip to next (Node, LT) pair
        prev_id = Some(current);
        current_id = next_id;
    }
    
    // Reached end of list - new_icr >= all nodes, insert at tail (safest)
    msg!("Reached end: new_icr {} >= all nodes, inserting at tail", new_icr);
    Ok((sorted_troves_state.tail, None))
}

/// Helper: Get ICR from LiquidityThreshold account
/// Expects the account to be a LiquidityThreshold PDA for the given owner
fn get_icr_from_account(account: &AccountInfo, expected_owner: Pubkey) -> Result<u64> {
    // Deserialize LiquidityThreshold account
    let threshold_data = account.try_borrow_data()?;
    let threshold = LiquidityThreshold::try_deserialize(&mut &threshold_data[8..])?;
    
    // Verify it's for the correct owner
    require!(
        threshold.owner == expected_owner,
        AerospacerProtocolError::InvalidList
    );
    
    Ok(threshold.ratio)
}
