use anchor_lang::prelude::*;

/// Sorted Troves management for efficient liquidation
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
    
    /// Insert a trove into the sorted list
    pub fn insert(&mut self, trove_key: Pubkey, _collateral_ratio: u64) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you would maintain a sorted linked list
        // based on collateral ratio (ascending order for liquidation)
        
        if self.head.is_none() {
            self.head = Some(trove_key);
            self.tail = Some(trove_key);
        } else {
            // For now, just append to the end
            // In a real implementation, you would insert in sorted order
            self.tail = Some(trove_key);
        }
        
        self.size = self.size.checked_add(1).ok_or(ErrorCode::Overflow)?;
        Ok(())
    }
    
    /// Remove a trove from the sorted list
    pub fn remove(&mut self, _trove_key: Pubkey) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you would properly remove from the linked list
        
        if self.size > 0 {
            self.size = self.size.checked_sub(1).ok_or(ErrorCode::Overflow)?;
        }
        
        if self.size == 0 {
            self.head = None;
            self.tail = None;
        }
        
        Ok(())
    }
    
    /// Get troves that can be liquidated
    pub fn get_liquidatable_troves(&self, _max_count: u32) -> Vec<Pubkey> {
        let result = Vec::new();
        
        // This is a simplified implementation
        // In a real implementation, you would traverse the sorted list
        // and return troves below the minimum collateral ratio
        
        // For now, return empty vector
        result
    }
    
    /// Check if trove exists in the list
    pub fn contains(&self, _trove_key: Pubkey) -> bool {
        // This is a simplified implementation
        // In a real implementation, you would traverse the list
        false
    }
    
    /// Get the size of the list
    pub fn len(&self) -> u32 {
        self.size
    }
    
    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow occurred")]
    Overflow,
} 