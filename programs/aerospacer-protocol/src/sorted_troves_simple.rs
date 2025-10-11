use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

/// Simplified sorted troves management using Anchor Account pattern
/// This module implements a doubly-linked list sorted by ICR (Individual Collateral Ratio)
/// Lower ICR = riskier trove = earlier in the list (head is riskiest)

/// Initialize the sorted troves state
pub fn initialize_sorted_troves(sorted_troves_state: &mut Account<SortedTrovesState>) -> Result<()> {
    sorted_troves_state.head = None;
    sorted_troves_state.tail = None;
    sorted_troves_state.size = 0;
    Ok(())
}

/// Insert a trove into the sorted list
/// This is called when a trove is opened
/// 
/// IMPORTANT: For Phase 2 (FIFO mode), this function only maintains the list STATE (head/tail/size)
/// but does NOT update neighbor node pointers (old_tail.next_id). This means:
/// - Size tracking is accurate
/// - Head/tail pointers are correct
/// - But linked list traversal via Node.prev_id/next_id won't work until Phase 3
/// 
/// Phase 3 will add remaining_accounts to pass neighbor nodes for full pointer updates.
pub fn insert_trove(
    sorted_troves_state: &mut Account<SortedTrovesState>,
    user_node: &mut Account<Node>,
    user: Pubkey,
    icr: u64,
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
    
    // FIFO append to tail - update state but NOT neighbor pointers
    // WARNING: This creates an inconsistent linked list (old_tail.next_id is not updated)
    // The list is "conceptually correct" (size/head/tail) but not traversable
    // Phase 3 will fix this by passing old_tail node via remaining_accounts
    if let Some(old_tail) = sorted_troves_state.tail {
        user_node.prev_id = Some(old_tail);
        // MISSING: old_tail.next_id = Some(user)  <-- requires old_tail node account
    }
    
    sorted_troves_state.tail = Some(user);
    sorted_troves_state.size += 1;
    
    msg!("Trove appended (FIFO, pointers incomplete): user={}, icr={}, size={}", user, icr, sorted_troves_state.size);
    msg!("WARNING: Linked list traversal not yet supported - requires Phase 3 neighbor node updates");
    Ok(())
}

/// Remove a trove from the sorted list
/// This is called when a trove is fully repaid
pub fn remove_trove(
    sorted_troves_state: &mut Account<SortedTrovesState>,
    user: Pubkey,
) -> Result<()> {
    require!(sorted_troves_state.size > 0, AerospacerProtocolError::TroveDoesNotExist);
    
    // Handle single trove case
    if sorted_troves_state.size == 1 {
        require!(
            sorted_troves_state.head == Some(user) && sorted_troves_state.tail == Some(user),
            AerospacerProtocolError::TroveDoesNotExist
        );
        
        sorted_troves_state.head = None;
        sorted_troves_state.tail = None;
        sorted_troves_state.size = 0;
        
        msg!("Last trove removed: user={}", user);
        return Ok(());
    }
    
    // For multi-trove case, we need neighbor nodes to update pointers
    // TODO Phase 3: Implement proper linked list removal with neighbor node accounts
    
    // For now, only handle head/tail removal (edge cases)
    if sorted_troves_state.head == Some(user) {
        // Removing head - next becomes new head
        // Note: We don't have next node to update its prev_id = None
        // This will be fixed in Phase 3
        sorted_troves_state.head = None; // Placeholder
    }
    
    if sorted_troves_state.tail == Some(user) {
        // Removing tail - prev becomes new tail
        // Note: We don't have prev node to update its next_id = None
        // This will be fixed in Phase 3
        sorted_troves_state.tail = None; // Placeholder
    }
    
    sorted_troves_state.size -= 1;
    
    msg!("Trove removed: user={}, remaining_size={}", user, sorted_troves_state.size);
    Ok(())
}

/// Reinsert a trove with new ICR (used when ICR changes)
/// This is called when collateral is added/removed or debt changes
pub fn reinsert_trove(
    sorted_troves_state: &mut Account<SortedTrovesState>,
    user_node: &mut Account<Node>,
    user: Pubkey,
    new_icr: u64,
) -> Result<()> {
    // For now, skip actual reinsertion since it requires neighbor nodes
    // TODO Phase 3: Implement remove + insert with ICR-based positioning
    
    msg!("Trove reinsertion requested: user={}, new_icr={} (STUB)", user, new_icr);
    Ok(())
}

/// Get the first (riskiest) trove
pub fn get_first_trove(sorted_troves_state: &Account<SortedTrovesState>) -> Option<Pubkey> {
    sorted_troves_state.head
}

/// Get the last (safest) trove
pub fn get_last_trove(sorted_troves_state: &Account<SortedTrovesState>) -> Option<Pubkey> {
    sorted_troves_state.tail
}
