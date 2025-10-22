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
    
    user_node.id = user;
    user_node.prev_id = None;
    user_node.next_id = None;

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
    
    // PRODUCTION SAFETY: Update list state with overflow protection
    sorted_troves_state.size = sorted_troves_state.size.checked_add(1)
        .ok_or(AerospacerProtocolError::OverflowError)?;
    
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
    
    // PRODUCTION SAFETY: Update list state with underflow protection
    sorted_troves_state.size = sorted_troves_state.size.checked_sub(1)
        .ok_or(AerospacerProtocolError::OverflowError)?;
    
    msg!("Trove removed: user={}, size={}", user, sorted_troves_state.size);
    Ok(())
}

/// Reinsert a trove when its ICR changes (e.g., after borrowing more or adding collateral)
/// **5% Threshold**: Only repositions if ICR changed by >=5% to avoid unnecessary gas costs
/// This performs: check threshold → remove from current position → find new position → insert
/// 
/// **remaining_accounts pattern (CORRECTED for alignment):**
/// - [0]: user_liquidity_threshold (for old ICR check)
/// - [1+]: old_neighbor_nodes (0-2 Node accounts for removal, NO LT accounts)
/// - [N+]: (node, lt) pairs for traversal from NEW head (includes duplicates of neighbors)
/// 
/// **Concrete examples:**
/// - Was head(ICR 120), next is B(130): [user_lt, B_node, B_node, B_lt, C_node, C_lt, ...]
///   → Remove: update B at [1], Traversal: starts at [2] with B_node (duplicate but aligned!)
/// - Was middle A→user→B: [user_lt, A_node, B_node, new_head_node, new_head_lt, ...]
///   → Remove: update A[1] and B[2], Traversal: starts at [3] (aligned!)
/// - Was tail with prev A: [user_lt, A_node, new_head_node, new_head_lt, ...]
///   → Remove: update A at [1], Traversal: starts at [2] (aligned!)
/// 
/// **KEY: Duplicates are intentional to ensure traversal starts at correct Node/LT pair alignment!**
pub fn reinsert_trove(
    sorted_troves_state: &mut SortedTrovesState,
    user_node: &mut Node,
    user: Pubkey,
    new_icr: u64,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    require!(new_icr > 0, AerospacerProtocolError::InvalidAmount);
    require!(sorted_troves_state.size > 0, AerospacerProtocolError::TroveDoesNotExist);
    
    // Single trove - no need to reinsert
    if sorted_troves_state.size == 1 {
        msg!("Single trove in list, no repositioning needed");
        return Ok(());
    }
    
    // Get old ICR from user's LiquidityThreshold to check if reinsert is needed
    require!(remaining_accounts.len() >= 1, AerospacerProtocolError::InvalidList);
    let old_icr = get_icr_from_account(&remaining_accounts[0], user)?;
    
    // Calculate ICR change percentage
    let icr_diff = if new_icr > old_icr {
        new_icr - old_icr
    } else {
        old_icr - new_icr
    };
    
    let threshold = old_icr / 20; // 5% of old_icr
    
    // Skip reinsert if change < 5% (optimization)
    if icr_diff < threshold {
        msg!("ICR change {} < 5% threshold {}, skipping reinsert", icr_diff, threshold);
        return Ok(());
    }
    
    msg!("Reinsert trove: user={}, old_icr={}, new_icr={}, change={}", 
         user, old_icr, new_icr, icr_diff);
    
    // Store current position
    let old_prev_id = user_node.prev_id;
    let old_next_id = user_node.next_id;
    
    // STEP 1: Remove from current position
    // Update neighbors' pointers to bypass this node
    // Determine traversal start index based on which neighbors exist
    let traversal_start_idx: usize;
    
    match (old_prev_id, old_next_id) {
        (Some(prev), Some(next)) => {
            // Both neighbors exist: indices 1 and 2
            require!(remaining_accounts.len() > 2, AerospacerProtocolError::InvalidList);
            
            // Update prev node
            let prev_account = &remaining_accounts[1];
            let mut prev_data = prev_account.try_borrow_mut_data()?;
            let mut prev_node = Node::try_deserialize(&mut &prev_data[8..])?;
            require!(prev_node.id == prev, AerospacerProtocolError::InvalidList);
            prev_node.next_id = old_next_id;
            let mut writer = &mut prev_data[8..];
            prev_node.try_serialize(&mut writer)?;
            drop(prev_data);
            
            // Update next node
            let next_account = &remaining_accounts[2];
            let mut next_data = next_account.try_borrow_mut_data()?;
            let mut next_node = Node::try_deserialize(&mut &next_data[8..])?;
            require!(next_node.id == next, AerospacerProtocolError::InvalidList);
            next_node.prev_id = old_prev_id;
            let mut writer = &mut next_data[8..];
            next_node.try_serialize(&mut writer)?;
            
            traversal_start_idx = 3; // Traversal starts after both neighbors
            msg!("Removed from middle: prev {} <-> next {}", prev, next);
        }
        (Some(prev), None) => {
            // Was tail: only prev exists at index 1
            require!(remaining_accounts.len() > 1, AerospacerProtocolError::InvalidList);
            
            let prev_account = &remaining_accounts[1];
            let mut prev_data = prev_account.try_borrow_mut_data()?;
            let mut prev_node = Node::try_deserialize(&mut &prev_data[8..])?;
            require!(prev_node.id == prev, AerospacerProtocolError::InvalidList);
            prev_node.next_id = None;
            let mut writer = &mut prev_data[8..];
            prev_node.try_serialize(&mut writer)?;
            
            sorted_troves_state.tail = Some(prev);
            traversal_start_idx = 2; // Traversal starts after prev
            msg!("Removed from tail: prev {} became new tail", prev);
        }
        (None, Some(next)) => {
            // Was head: only next exists at index 1
            require!(remaining_accounts.len() > 1, AerospacerProtocolError::InvalidList);
            
            let next_account = &remaining_accounts[1];
            let mut next_data = next_account.try_borrow_mut_data()?;
            let mut next_node = Node::try_deserialize(&mut &next_data[8..])?;
            require!(next_node.id == next, AerospacerProtocolError::InvalidList);
            next_node.prev_id = None;
            let mut writer = &mut next_data[8..];
            next_node.try_serialize(&mut writer)?;
            
            sorted_troves_state.head = Some(next);
            traversal_start_idx = 2; // Traversal starts after next
            msg!("Removed from head: next {} became new head", next);
        }
        (None, None) => {
            // Should not happen
            return Err(AerospacerProtocolError::InvalidList.into());
        }
    }
    
    // Temporarily decrement size for find_insert_position
    sorted_troves_state.size -= 1;
    
    // STEP 2: Find new position based on new ICR
    // Traversal accounts are properly aligned to Node/LT pairs from traversal_start_idx
    let traversal_accounts = &remaining_accounts[traversal_start_idx..];
    let (new_prev_id, new_next_id) = find_insert_position(
        sorted_troves_state,
        new_icr,
        traversal_accounts,
    )?;
    
    // STEP 3: Insert at new position
    // Update user node's pointers
    user_node.prev_id = new_prev_id;
    user_node.next_id = new_next_id;
    
    // Update new neighbors - search in ALL remaining_accounts starting from index 1
    // This ensures old neighbors are still available if trove stays in same area
    let search_accounts = &remaining_accounts[1..]; // Skip user_lt, search all Node accounts
    
    match (new_prev_id, new_next_id) {
        (None, Some(next)) => {
            // New head position - search for next node
            let mut found = false;
            for node_account in search_accounts.iter() {
                if let Ok(node_data) = node_account.try_borrow_data() {
                    if let Ok(node) = Node::try_deserialize(&mut &node_data[8..]) {
                        if node.id == next {
                            drop(node_data);
                            
                            let mut mut_data = node_account.try_borrow_mut_data()?;
                            let mut next_node = Node::try_deserialize(&mut &mut_data[8..])?;
                            next_node.prev_id = Some(user);
                            let mut writer = &mut mut_data[8..];
                            next_node.try_serialize(&mut writer)?;
                            
                            found = true;
                            break;
                        }
                    }
                }
            }
            require!(found, AerospacerProtocolError::InvalidList);
            
            sorted_troves_state.head = Some(user);
            msg!("Reinserted at head: {} -> {}", user, next);
        }
        (Some(prev), None) => {
            // New tail position - search for prev node
            let mut found = false;
            for node_account in search_accounts.iter() {
                if let Ok(node_data) = node_account.try_borrow_data() {
                    if let Ok(node) = Node::try_deserialize(&mut &node_data[8..]) {
                        if node.id == prev {
                            drop(node_data);
                            
                            let mut mut_data = node_account.try_borrow_mut_data()?;
                            let mut prev_node = Node::try_deserialize(&mut &mut_data[8..])?;
                            prev_node.next_id = Some(user);
                            let mut writer = &mut mut_data[8..];
                            prev_node.try_serialize(&mut writer)?;
                            
                            found = true;
                            break;
                        }
                    }
                }
            }
            require!(found, AerospacerProtocolError::InvalidList);
            
            sorted_troves_state.tail = Some(user);
            msg!("Reinserted at tail: {} -> {}", prev, user);
        }
        (Some(prev), Some(next)) => {
            // Middle position - search for both neighbors
            let mut found_prev = false;
            let mut found_next = false;
            
            for node_account in search_accounts.iter() {
                if let Ok(node_data) = node_account.try_borrow_data() {
                    if let Ok(node) = Node::try_deserialize(&mut &node_data[8..]) {
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
                }
            }
            
            require!(found_prev && found_next, AerospacerProtocolError::InvalidList);
            msg!("Reinserted in middle: {} -> {} -> {}", prev, user, next);
        }
        (None, None) => {
            // Should not happen
            return Err(AerospacerProtocolError::InvalidList.into());
        }
    }
    
    // Restore size
    sorted_troves_state.size += 1;
    
    msg!("Trove repositioned: user={}, new_icr={}", user, new_icr);
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

/// Get all liquidatable troves (ICR < liquidation_threshold) by walking the sorted list
/// Starts from head (riskiest) and walks until ICR >= threshold (sorted list optimization)
/// 
/// # Arguments
/// * `sorted_troves_state` - The sorted troves state containing head/tail/size
/// * `liquidation_threshold` - The ICR threshold below which troves are liquidatable (typically 110)
/// * `remaining_accounts` - Node and LT account pairs for traversal [node1, lt1, node2, lt2, ...]
/// 
/// # Returns
/// Vec<Pubkey> of liquidatable trove owners, ordered from riskiest to safest
/// 
/// # Remaining Accounts Pattern
/// The caller must pass ALL troves' Node and LiquidityThreshold accounts in traversal order:
/// - [0]: First Node account (head)
/// - [1]: First LiquidityThreshold account
/// - [2]: Second Node account
/// - [3]: Second LiquidityThreshold account
/// - ...and so on
/// 
/// The function will stop early once it finds ICR >= threshold (sorted list optimization)
pub fn get_liquidatable_troves(
    sorted_troves_state: &SortedTrovesState,
    liquidation_threshold: u64,
    remaining_accounts: &[AccountInfo],
) -> Result<Vec<Pubkey>> {
    let mut liquidatable = Vec::new();
    
    // Empty list - no liquidatable troves
    if sorted_troves_state.size == 0 {
        msg!("Sorted list empty - no troves to liquidate");
        return Ok(liquidatable);
    }
    
    // Start from head (riskiest troves)
    let mut current_id = sorted_troves_state.head;
    let mut account_idx = 0;
    
    msg!("Walking sorted list from head to find liquidatable troves (ICR < {})", liquidation_threshold);
    
    while let Some(current) = current_id {
        // Validate we have enough remaining_accounts
        if account_idx + 1 >= remaining_accounts.len() {
            msg!("Not enough accounts for traversal at index {}", account_idx);
            break;
        }
        
        // Get Node and LiquidityThreshold accounts for current trove
        let node_account = &remaining_accounts[account_idx];
        let lt_account = &remaining_accounts[account_idx + 1];
        
        // Deserialize Node to get next_id and verify identity
        let node_data = node_account.try_borrow_data()?;
        let node = Node::try_deserialize(&mut &node_data[8..])?;
        require!(node.id == current, AerospacerProtocolError::InvalidList);
        
        // Get ICR from LiquidityThreshold account
        let current_icr = get_icr_from_account(lt_account, current)?;
        
        msg!("Checking trove {}: ICR = {}", current, current_icr);
        
        // Check if liquidatable (ICR < threshold)
        if current_icr < liquidation_threshold {
            liquidatable.push(current);
            msg!("  -> Liquidatable (ICR {} < threshold {})", current_icr, liquidation_threshold);
        } else {
            // Sorted list optimization: once we find ICR >= threshold, all remaining are safe
            msg!("  -> Safe (ICR {} >= threshold {}). Stopping traversal (sorted list)", current_icr, liquidation_threshold);
            break;
        }
        
        // Move to next node
        account_idx += 2; // Skip to next (Node, LT) pair
        current_id = node.next_id;
    }
    
    msg!("Found {} liquidatable troves", liquidatable.len());
    Ok(liquidatable)
}
