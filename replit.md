# Aerospacer Protocol - Replit Development Environment

## Overview
The Aerospacer Protocol is a decentralized lending platform on Solana, enabling Collateralized Debt Positions (CDPs), stablecoin (aUSD) minting, and an automated liquidation system. It integrates Pyth Network for price feeds and features a robust fee distribution mechanism. The project aims to provide a secure and efficient on-chain lending solution within the Solana ecosystem, offering a new primitive for decentralized finance.

## User Preferences
*This section will be updated as you work with the project*

## System Architecture

**Core Programs:**
The project consists of three primary Solana smart contract programs built with Anchor v0.28.0 in Rust:
1.  **aerospacer-protocol**: Manages core lending logic, including CDPs, stablecoin minting, and liquidation.
2.  **aerospacer-oracle**: Handles price feed management, primarily integrating with the Pyth Network.
3.  **aerospacer-fees**: Manages fee collection and distribution.

**UI/UX Decisions:**
The design supports transparent and auditable on-chain interactions, with all state changes and operations publicly verifiable on the Solana blockchain.

**Technical Implementations & Feature Specifications:**
*   **Collateralized Debt Positions (CDPs)**: Users can lock collateral to mint aUSD stablecoins.
*   **Stablecoin (aUSD) Minting**: Supports the minting of its native stablecoin, aUSD.
*   **Automated Liquidation System**: Ensures protocol solvency by liquidating undercollateralized positions.
*   **Stability Pool with Snapshot-Based Distribution**: Implements Liquity's Product-Sum algorithm for fair and exploit-resistant reward distribution.
*   **Fee Distribution Mechanism**: A dual-mode system that distributes fees to either the stability pool or 50/50 to specified fee addresses.
*   **Oracle Integration**: Utilizes Pyth Network for real-time price feeds for all collateral assets, with dynamic collateral discovery via CPI.
*   **Cross-Program Communication (CPI)**: Extensive use of CPI for secure and atomic interactions between sub-programs.
*   **SPL Token Integration**: Full support for Solana Program Library (SPL) tokens for collateral and stablecoin operations.
*   **Sorted Troves**: Implemented as a doubly-linked list for efficient management of CDPs, supporting ICR-based positioning and auto-discovery of liquidatable troves.
*   **Individual Collateral Ratio (ICR)**: Comprehensive, real-time ICR calculations are implemented across the protocol, supporting multi-collateral types and ensuring solvency checks.
*   **Redemption System**: Integrates with the sorted troves list, supporting both full and partial redemptions.
*   **Admin State Recovery**: Includes `reset_sorted_troves` instruction for recovering from corrupted sorted troves state on devnet.

**System Design Choices:**
*   **Anchor Framework**: Used for Solana smart contract development.
*   **Rust & TypeScript**: Rust for on-chain programs and TypeScript for off-chain tests and interactions.
*   **Modular Architecture**: Separation of concerns into distinct programs (`protocol`, `oracle`, `fees`).
*   **Security Features**: Includes safe math operations, access control, input validation, atomic state consistency, PDA validation, and optimized for Solana BPF stack limits.
*   **Two-Instruction Architecture for Liquidation**: Separates data traversal from execution to optimize account ordering.
*   **Vault Signing Architecture**: All PDA vault authorities correctly sign CPIs using `invoke_signed`.
*   **BPF Stack Optimization**: Use of `UncheckedAccount` pattern to mitigate Solana BPF stack limits during account deserialization.

## External Dependencies

*   **Solana Blockchain**: The foundational blockchain layer.
*   **Anchor Framework**: Solana smart contract development framework.
*   **Pyth Network**: Used by the `aerospacer-oracle` program for real-time price feeds.
*   **Solana Program Library (SPL) Tokens**: Integrated for token operations within the protocol.
*   **Node.js & npm**: For running TypeScript tests and managing project dependencies.

---

## Recent Changes

**October 21, 2025 - Devnet State Corruption Fix: Complete Recovery Solution** ✅
- **ISSUE**: Devnet sorted troves state corrupted (SortedTrovesState.size=1 but Node accounts have wrong discriminators)
  * Root Cause: Devnet accounts were reused/overwritten, making sorted list untraversable
  * Error: `AccountDiscriminatorMismatch` (3002) when passing Node accounts to contract
  
- **COMPLETE SOLUTION IMPLEMENTED**:
  * ✅ **NEW INSTRUCTION**: `reset_sorted_troves` admin instruction added to protocol program
    - Uses Anchor's `close = authority` constraint for safe PDA closing
    - Requires admin authority check via `state.authority`
    - Refunds lamports to admin wallet
    - Next `openTrove` automatically reinitializes fresh state
  * ✅ **CLEANUP SCRIPT**: `scripts/close-sorted-troves-devnet.ts`
    - Calls `reset_sorted_troves` instruction on devnet
    - Safety preflight checks and 3-second countdown
    - Clear error messages and troubleshooting
    - Validates admin authority
  * ✅ **DETECTION**: Enhanced `getExistingTrovesAccounts` with discriminator validation
    - Validates first 8 bytes match expected account types
    - Throws error when corruption detected (cannot return empty array)
    - Provides clear remediation instructions
    
- **RECOVERY PROCESS**:
  1. Build and redeploy: `anchor build && anchor deploy --provider.cluster devnet`
  2. Run cleanup: `npx ts-node scripts/close-sorted-troves-devnet.ts`
  3. Run tests: Next `openTrove` creates fresh sorted list
  
- **DOCUMENTATION**: Step-by-step guide in DEVNET_COLLATERAL_SETUP.md
  * "Fixing Corrupted Sorted Troves State" section with 3-step recovery
  * Updated troubleshooting to reference cleanup script
  * Verification steps included
  
- **ARCHITECT REVIEW**: Approved ✅
  * Security: Admin access properly gated
  * Safety: Anchor close constraint handles lamports correctly  
  * Design: Follows Solana best practices for PDA management
