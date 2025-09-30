use anchor_lang::prelude::*;
use crate::state::*;
use crate::utils::*;

/// Node structure for sorted troves linked list (equivalent to INJECTIVE's Node)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Node {
    pub prev_id: Option<Pubkey>,
    pub next_id: Option<Pubkey>,
}

/// Sorted Troves management for efficient liquidation (equivalent to INJECTIVE's sorted_troves)
pub struct SortedTroves {
    pub head: Option<Pubkey>,
    pub tail: Option<Pubkey>,
    pub size: u32,
}

impl SortedTroves {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            size: 0,
        }
    }
    
    /// Insert a trove into the sorted list (equivalent to INJECTIVE's insert_trove)
    pub fn insert(
        &mut self,
        trove_key: Pubkey,
        collateral_ratio: u64,
        prev_id: Option<Pubkey>,
        next_id: Option<Pubkey>,
    ) -> Result<()> {
        // Validate node position
        if !self.check_node_position(collateral_ratio, prev_id, next_id)? {
            let (new_prev_id, new_next_id) = self.find_insert_location(collateral_ratio, prev_id, next_id)?;
            return self.insert(trove_key, collateral_ratio, new_prev_id, new_next_id);
        }
        
        // Create the node
        let node = Node {
            prev_id: prev_id.clone(),
            next_id: next_id.clone(),
        };
        
        // Update linked list
        if prev_id.is_none() && next_id.is_none() {
            // First trove
            self.head = Some(trove_key);
            self.tail = Some(trove_key);
        } else if prev_id.is_none() {
            // Insert at head
            if let Some(head_key) = self.head {
                self.update_node(trove_key, false, Some(head_key))?;
                self.update_node(head_key, true, Some(trove_key))?;
                self.head = Some(trove_key);
            }
        } else if next_id.is_none() {
            // Insert at tail
            if let Some(tail_key) = self.tail {
                self.update_node(trove_key, true, Some(tail_key))?;
                self.update_node(tail_key, false, Some(trove_key))?;
                self.tail = Some(trove_key);
            }
        } else {
            // Insert in middle
            if let (Some(prev), Some(next)) = (prev_id, next_id) {
                self.update_node(trove_key, true, Some(prev))?;
                self.update_node(trove_key, false, Some(next))?;
                self.update_node(prev, false, Some(trove_key))?;
                self.update_node(next, true, Some(trove_key))?;
            }
        }
        
        self.size = self.size.checked_add(1).ok_or(ErrorCode::Overflow)?;
        
        Ok(())
    }
    
    /// Reinsert a trove (equivalent to INJECTIVE's reinsert_trove)
    pub fn reinsert(
        &mut self,
        trove_key: Pubkey,
        collateral_ratio: u64,
        prev_id: Option<Pubkey>,
        next_id: Option<Pubkey>,
    ) -> Result<()> {
        // Remove existing trove
        self.remove(trove_key)?;
        
        // Insert with new ratio
        self.insert(trove_key, collateral_ratio, prev_id, next_id)
    }
    
    /// Remove a trove from the sorted list (equivalent to INJECTIVE's remove_trove)
    pub fn remove(&mut self, trove_key: Pubkey) -> Result<()> {
        if self.size == 0 {
            return Ok(());
        }
        
        if self.size == 1 {
            self.head = None;
            self.tail = None;
        } else {
            // Get node info and update neighbors
            // This is a simplified version - in real implementation you'd need to store nodes
            if let Some(head) = self.head {
                if head == trove_key {
                    // Update head
                    // In real implementation, you'd get the next node and update it
                    self.head = None; // Simplified
                }
            }
            
            if let Some(tail) = self.tail {
                if tail == trove_key {
                    // Update tail
                    // In real implementation, you'd get the prev node and update it
                    self.tail = None; // Simplified
                }
            }
        }
        
        self.size = self.size.checked_sub(1).ok_or(ErrorCode::Overflow)?;
        
        Ok(())
    }
    
    /// Get first trove (equivalent to INJECTIVE's get_first_trove)
    pub fn get_first_trove(&self) -> Option<Pubkey> {
        self.head
    }
    
    /// Get last trove (equivalent to INJECTIVE's get_last_trove)
    pub fn get_last_trove(&self) -> Option<Pubkey> {
        self.tail
    }
    
    /// Get next trove (equivalent to INJECTIVE's get_next_trove)
    pub fn get_next_trove(&self, current: Pubkey) -> Option<Pubkey> {
        // In real implementation, you'd look up the node and return next_id
        // For now, return None as placeholder
        None
    }
    
    /// Get previous trove (equivalent to INJECTIVE's get_prev_trove)
    pub fn get_prev_trove(&self, current: Pubkey) -> Option<Pubkey> {
        // In real implementation, you'd look up the node and return prev_id
        // For now, return None as placeholder
        None
    }
    
    /// Check node position validity (equivalent to INJECTIVE's check_node_position)
    fn check_node_position(
        &self,
        collateral_ratio: u64,
        prev_id: Option<Pubkey>,
        next_id: Option<Pubkey>,
    ) -> Result<bool> {
        if prev_id.is_none() && next_id.is_none() {
            return Ok(self.size == 0);
        }
        
        if prev_id.is_none() {
            if let Some(next) = next_id {
                return Ok(self.head == Some(next) && collateral_ratio >= self.get_trove_ratio(next)?);
            }
        }
        
        if next_id.is_none() {
            if let Some(prev) = prev_id {
                return Ok(self.tail == Some(prev) && collateral_ratio <= self.get_trove_ratio(prev)?);
            }
        }
        
        // Check middle position
        if let (Some(prev), Some(next)) = (prev_id, next_id) {
            // In real implementation, you'd verify the nodes are connected and ratios are correct
            return Ok(true); // Simplified
        }
        
        Ok(false)
    }
    
    /// Find insert location (equivalent to INJECTIVE's find_insert_location)
    fn find_insert_location(
        &self,
        collateral_ratio: u64,
        mut prev_id: Option<Pubkey>,
        mut next_id: Option<Pubkey>,
    ) -> Result<(Option<Pubkey>, Option<Pubkey>)> {
        // Validate and adjust prev_id
        if let Some(prev) = prev_id {
            if !self.contains(prev) || collateral_ratio > self.get_trove_ratio(prev)? {
                prev_id = None;
            }
        }
        
        // Validate and adjust next_id
        if let Some(next) = next_id {
            if !self.contains(next) || collateral_ratio < self.get_trove_ratio(next)? {
                next_id = None;
            }
        }
        
        // Find proper location
        if prev_id.is_none() && next_id.is_none() {
            if let Some(head) = self.head {
                return self.descend_list(collateral_ratio, head);
            }
        }
        
        if prev_id.is_none() {
            if let Some(next) = next_id {
                return self.ascend_list(collateral_ratio, next);
            }
        }
        
        if next_id.is_none() {
            if let Some(prev) = prev_id {
                return self.descend_list(collateral_ratio, prev);
            }
        }
        
        Ok((prev_id, next_id))
    }
    
    /// Ascend list to find position (equivalent to INJECTIVE's ascend_list)
    fn ascend_list(&self, collateral_ratio: u64, start: Pubkey) -> Result<(Option<Pubkey>, Option<Pubkey>)> {
        // Simplified implementation
        // In real implementation, you'd traverse the list upward
        Ok((None, Some(start)))
    }
    
    /// Descend list to find position (equivalent to INJECTIVE's descend_list)
    fn descend_list(&self, collateral_ratio: u64, start: Pubkey) -> Result<(Option<Pubkey>, Option<Pubkey>)> {
        // Simplified implementation
        // In real implementation, you'd traverse the list downward
        Ok((Some(start), None))
    }
    
    /// Update node connections (equivalent to INJECTIVE's update_node)
    fn update_node(&mut self, id: Pubkey, is_prev: bool, value: Option<Pubkey>) -> Result<()> {
        // In real implementation, you'd update the stored node
        // For now, this is a placeholder
        Ok(())
    }
    
    /// Check if trove exists in the list
    pub fn contains(&self, _trove_key: Pubkey) -> bool {
        // In real implementation, you'd check if the trove exists
        // For now, return true as placeholder
        true
    }
    
    /// Get trove ratio (placeholder - in real implementation you'd look this up)
    fn get_trove_ratio(&self, _trove_key: Pubkey) -> Result<u64> {
        // In real implementation, you'd look up the trove's collateral ratio
        // For now, return a default value
        Ok(10000) // 100% as placeholder
    }
    
    /// Get the size of the list
    pub fn len(&self) -> u32 {
        self.size
    }
    
    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    
    /// Get liquidatable troves (equivalent to INJECTIVE's liquidation logic)
    pub fn get_liquidatable_troves(&self, max_count: u32) -> Vec<Pubkey> {
        let mut result = Vec::new();
        let mut current = self.head;
        let mut count = 0;
        
        while current.is_some() && count < max_count {
            if let Some(trove_key) = current {
                // In real implementation, you'd check if the trove is liquidatable
                // For now, add all troves as placeholder
                result.push(trove_key);
                count += 1;
                
                // Move to next trove
                current = self.get_next_trove(trove_key);
            }
        }
        
        result
    }
}

/// Initialize sorted troves (equivalent to INJECTIVE's initialize_sorted_troves)
pub fn initialize_sorted_troves() -> Result<SortedTroves> {
    Ok(SortedTroves::new())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
} 