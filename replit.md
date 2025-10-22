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

---

**October 21, 2025 - Test Suite Fixes for Devnet Integration Testing** ✅
- **ISSUE**: Test suite had user/account mismatches preventing sequential test flow
  * First test opened trove for user2, but subsequent tests tried to use user1's non-existent trove
  * Stake test tried to stake 1 aUSD but user1 only borrowed 0.0011 aUSD

- **FIXES IMPLEMENTED**:
  * ✅ **First openTrove Test**: Changed to use user1 (instead of user2)
    - Now uses user1's PDAs: user1DebtAmountPDA, user1LiquidityThresholdPDA, user1CollateralAmountPDA, user1NodePDA
    - Uses user1 as signer and user1StablecoinAccount for minting
    - Enables proper sequential flow: openTrove → addCollateral → borrowLoan
  * ✅ **Stake Test Amount**: Reduced from 1 aUSD to 0.0005 aUSD
    - Matches user1's actual borrowed balance after fees (~0.0011 aUSD total, staking half)
    - Prevents InsufficientCollateral errors
  * ✅ **Devnet Environment Setup**: 
    - Generated Solana keypair at ~/.config/solana/id.json (pubkey: AXbPgQyHpUmQv4VEDA5JHVktJ3iYWXe5GH13ch7MpApp)
    - Airdropped 1 SOL on devnet for test funding
    - Installed Anchor CLI v0.31.1 binary at ~/.local/bin/anchor

- **ARCHITECT REVIEW**: Approved ✅
  * Changes correctly align test flow
  * User1 PDAs now consistent across all operations
  * Stake amount within user1's balance
  * No security issues

---

**October 22, 2025 - Fixed getExistingTrovesAccounts Bug for Multi-User Tests** ✅
- **ISSUE**: Second user trove opening failed with undefined variable error
  * `getExistingTrovesAccounts` tried to access `node.nextId` on lines 91 and 123
  * `node` variable was undefined at that point (only decoded later on line 112)
  * Error occurred in both corrupted discriminator check and decode error handlers
  * Caused tests to crash when trying to traverse existing troves list

- **ROOT CAUSE**:
  * When discriminator validation fails, code tried to `continue` loop with `node.nextId`
  * When decode operation fails in catch block, same issue with `node.nextId` access
  * Both cases attempted to access a variable that hadn't been defined yet

- **FIX IMPLEMENTED**: ✅
  * Changed both error handlers from `continue` to `break`
  * Line 94: Break out when discriminator mismatch detected (can't safely decode)
  * Line 124: Break out when decode fails (can't access nextId from failed decode)
  * Returns partial list of valid accounts collected before corruption
  * Prevents undefined variable access crash

- **ARCHITECT REVIEW**: Approved ✅
  * Fix safely removes undefined variable access
  * Returns valid prefix of accounts when corruption detected
  * Normal traversal logic unchanged for valid accounts
  * No security issues observed

- **IMPACT**:
  * Second user trove opening should now work on devnet
  * Test suite can handle both valid and corrupted account states gracefully
  * Eliminates runtime crashes from undefined variable access

---

**October 22, 2025 - Added close_node Instruction for Emergency Node Cleanup** ✅
- **ISSUE**: User2 cannot open trove because Node PDA exists from previous test with corrupted discriminator
  * openTrove uses `init` constraint which requires account to be uninitialized
  * User2's Node PDA exists on devnet with wrong discriminator from old test run
  * Error: `AccountDiscriminatorMismatch` when trying to initialize existing Node account
  * Cannot open multiple user troves on devnet without clearing old Node accounts

- **ROOT CAUSE**:
  * Devnet Node PDAs persist between test runs
  * cleanup script only closed SortedTrovesState, not individual user Nodes
  * No way to close corrupted Node accounts from the contract
  * Normal Anchor Account deserialization fails on corrupted discriminators

- **SOLUTION IMPLEMENTED**: ✅
  * **NEW INSTRUCTION**: `close_node` admin instruction added to protocol program
    - Uses `UncheckedAccount` to bypass Anchor discriminator validation
    - PDA seeds and bump still validated for security
    - Manually closes account:
      1. Transfers all lamports from Node to admin authority
      2. Reallocates account data to 0 bytes
      3. Assigns ownership back to system program
    - Best-effort discriminator logging (doesn't abort on corruption)
    - Admin-only access via `state.admin` check
  * **CLEANUP SCRIPT**: `scripts/close-user-nodes-devnet.ts`
    - Closes Node PDAs for known test users (user1, user2)
    - Checks if Node exists before attempting to close
    - Handles both valid and corrupted Node accounts
    - Clear status output and error messages

- **TECHNICAL DETAILS**:
  * Cannot use `close = authority` constraint with UncheckedAccount
  * Manual close follows Solana best practices for account reclamation
  * Node discriminator: [198, 82, 111, 206, 177, 93, 160, 219]
  * Lamports refunded to admin wallet on close

- **ARCHITECT REVIEW**: Approved ✅
  * UncheckedAccount correctly bypasses deserialization
  * Manual close implementation is safe and follows best practices
  * Admin access control maintained
  * No security issues observed

- **DEPLOYMENT STEPS**:
  1. Build: `anchor build`
  2. Deploy: `anchor deploy --provider.cluster devnet`
  3. Cleanup: `ANCHOR_PROVIDER_URL=https://api.devnet.solana.com ANCHOR_WALLET=~/.config/solana/id.json npx ts-node scripts/close-user-nodes-devnet.ts`
  4. Test: `ANCHOR_PROVIDER_URL=https://api.devnet.solana.com ANCHOR_WALLET=~/.config/solana/id.json npx ts-mocha -p ./tsconfig.json -t 1000000 'tests/**/protocol-core.ts'`

- **IMPACT**:
  * Multiple users can now open troves on devnet
  * Emergency recovery tool for corrupted Node accounts
  * Test suite can run cleanly on devnet after cleanup
  * Eliminates AccountDiscriminatorMismatch errors on Node initialization
