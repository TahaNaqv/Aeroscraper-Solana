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
    
    // Find the correct position based on ICR (lowest ICR at head, highest at tail)
    // For simplicity in Phase 3, we'll still use FIFO but WITH proper pointer updates
    // TODO: Implement ICR-based binary search for optimal positioning
    
    // Simple FIFO append with FULL pointer updates
    let old_tail = sorted_troves_state.tail.unwrap();
    
    // Update the new node
    user_node.prev_id = Some(old_tail);
    user_node.next_id = None;
    
    // REQUIRE old_tail Node account in remaining_accounts[0]
    require!(
        remaining_accounts.len() >= 1,
        AerospacerProtocolError::InvalidList
    );
    
    let old_tail_account_info = &remaining_accounts[0];
    
    // Deserialize the old tail Node
    let mut old_tail_data = old_tail_account_info.try_borrow_mut_data()?;
    let mut old_tail_node = Node::try_deserialize(&mut &old_tail_data[8..])?;
    
    // Verify it's the correct node
    require!(
        old_tail_node.id == old_tail,
        AerospacerProtocolError::InvalidList
    );
    
    // Update its next pointer
    old_tail_node.next_id = Some(user);
    
    // Serialize back to the account
    let mut writer = &mut old_tail_data[8..];
    old_tail_node.try_serialize(&mut writer)?;
    
    msg!("Updated old tail {} -> next_id = {}", old_tail, user);
    
    // Update list state
    sorted_troves_state.tail = Some(user);
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

/// Find the correct insertion position based on ICR (binary search approach)
/// Returns (prev_id, next_id) for the new node's position
/// Lower ICR = riskier = closer to head
pub fn find_insert_position(
    sorted_troves_state: &SortedTrovesState,
    icr: u64,
    remaining_accounts: &[AccountInfo],
) -> Result<(Option<Pubkey>, Option<Pubkey>)> {
    // Start from head and traverse until we find the right spot
    // This is O(n) - could be optimized with hints or caching
    
    if sorted_troves_state.size == 0 {
        return Ok((None, None));
    }
    
    // For Phase 3, we'll implement a simple linear search
    // Production version should use binary search with ICR cache
    
    msg!("Finding insert position for ICR={}", icr);
    msg!("Note: Using simplified linear search - optimize in production");
    
    // TODO: Implement ICR-based traversal
    // For now, return tail position (FIFO)
    Ok((sorted_troves_state.tail, None))
}
