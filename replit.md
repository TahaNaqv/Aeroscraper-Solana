# Aerospacer Protocol - Replit Development Environment

## Overview
The Aerospacer Protocol is a decentralized lending platform on Solana. Its core purpose is to enable Collateralized Debt Positions (CDPs), mint the aUSD stablecoin, and manage an automated liquidation system. It integrates with Pyth Network for price feeds and features a robust fee distribution mechanism. The project aims to provide a secure and efficient on-chain lending solution within the Solana ecosystem, introducing a new primitive for decentralized finance.

## User Preferences
*This section will be updated as you work with the project*

## Recent Changes

### Off-Chain Sorting Architecture (2025-01-24)

**CRITICAL SCALABILITY FIX**: Resolved transaction size limit that prevented production deployment beyond 3-4 troves.

**Problem Identified:**
- On-chain linked list required ALL nodes in `remainingAccounts` for traversal
- Transaction size: 3-4 troves = 1287 bytes > 1232 byte limit ❌
- Protocol could not scale beyond handful of users

**Solution Implemented:**
- **Removed on-chain storage**: Deleted `Node` and `SortedTrovesState` structs (eliminated ~300 lines of state management)
- **Simplified sorted_troves.rs**: Reduced from 668 lines → 217 lines (68% reduction)
- **Off-chain sorting + on-chain validation**: Client fetches all troves via RPC, sorts by ICR, passes only 2-3 neighbor hints (~6-9 accounts = ~200 bytes)
- **Contract validates ordering**: Checks `prev_icr <= trove_icr <= next_icr` without storing anything

**Benefits:**
- ✅ No transaction size limits (can handle 1000+ troves)
- ✅ Reduced on-chain storage costs (no Node accounts)
- ✅ Simpler contract logic (validation-only)
- ✅ Flexible client-side sorting strategies

**Architecture Changes:**
- **Removed**: `insert_trove()`, `remove_trove()`, `reinsert_trove()`, `find_insert_position()` functions
- **Kept**: `get_liquidatable_troves()` (simplified to accept pre-sorted list via remainingAccounts)
- **New**: `validate_icr_ordering()` function for neighbor validation
- **New**: `tests/trove-indexer.ts` - Client-side utility for fetching, sorting, and finding neighbors
- **Updated**: All instructions (open_trove, close_trove, add_collateral, etc.) to remove linked list operations
- **Removed**: Admin cleanup instructions (`reset_sorted_troves`, `close_node`) - no longer needed

**Testing Impact:**
- All tests updated to use off-chain sorting pattern
- Helper functions added to `protocol-test-utils.ts`: `fetchAllTrovesSimple()`, `sortTrovesByICR()`, `findNeighborAccountsSimple()`

### Protocol Fee Integration Tests Fixed (2025-01-23)

Implemented comprehensive fee integration tests in `tests/protocol-fees-integration.ts` following code reuse principles:

**Code Reuse Implementation:**
- Migrated from duplicate setup code to using `setupTestEnvironment()` from `protocol-test-utils.ts`
- Follows same pattern as `protocol-oracle-integration.ts` (established reference implementation)
- Uses shared helper functions: `createTestUser()`, `openTroveForUser()`, `derivePDAs()`

**Test Coverage (6 tests total):**
1. **Test 8.1 - Fee Distribution via CPI**: Opens real trove, verifies fee CPI works, checks protocol state references correct fee program
2. **Test 8.2 - Protocol Fee Calculation (5%)**: Opens trove with 1000 aUSD loan, verifies 5% fee calculation is correct
3. **Test 8.3 - Stability Pool Mode Distribution**: Explicitly toggles to enable stability pool mode, verifies 100% of fees distributed to stability pool (strict validation with ±1 lamport tolerance), restores original mode
4. **Test 8.4 - Treasury Mode Distribution**: Explicitly toggles to disable stability pool mode, verifies 100% of fees split 50/50 to treasury addresses (strict validation with ±1 lamport tolerance), restores original mode
5. **Test 8.5 - Fee State Validation**: Architectural verification - validates fee_state_addr in protocol state prevents fake fee contract injection
6. **Test 8.6 - Fee Account Owner Validation**: Architectural verification - validates payer token account ownership enforced at runtime

**Key Implementation Details:**
- **Deterministic Mode Testing**: Tests 8.3 and 8.4 explicitly toggle `isStakeEnabled` flag via `toggleStakeContract()` instruction to force specific distribution modes, then restore original state
- **Strict 100% Validation**: Uses BigInt comparisons with ±1 lamport tolerance to verify exact fee amounts distributed (not just >= checks)
- **BigInt-Safe Comparisons**: All token balance comparisons use BigInt throughout to avoid Number precision loss for large loan amounts
- **50/50 Split Validation**: Treasury mode uses BigInt-safe 1% tolerance check for split verification
- **Fee Account Architecture**: Confirmed stabilityPoolTokenAccount, feeAddress1TokenAccount, feeAddress2TokenAccount are ATAs (Associated Token Accounts), not PDAs

## System Architecture

**Core Programs:**
The project uses Anchor v0.28.0 in Rust and consists of three main Solana smart contract programs:
1.  **aerospacer-protocol**: Manages core lending logic, including CDPs, stablecoin minting, and liquidation.
2.  **aerospacer-oracle**: Handles price feed management, primarily integrating with the Pyth Network.
3.  **aerospacer-fees**: Manages fee collection and distribution.

**UI/UX Decisions:**
The design prioritizes transparent and auditable on-chain interactions, ensuring all state changes and operations are publicly verifiable on the Solana blockchain.

**Technical Implementations & Feature Specifications:**
*   **Collateralized Debt Positions (CDPs)**: Allows users to lock collateral to mint aUSD stablecoins.
*   **Stablecoin (aUSD) Minting**: Supports the minting of its native stablecoin, aUSD.
*   **Automated Liquidation System**: Ensures protocol solvency by liquidating undercollateralized positions.
*   **Stability Pool**: Implements Liquity's Product-Sum algorithm for reward distribution.
*   **Fee Distribution Mechanism**: A dual-mode system for distributing fees to the stability pool or splitting them between specified addresses.
*   **Oracle Integration**: Uses Pyth Network for real-time price feeds for all collateral assets, with dynamic collateral discovery via CPI.
*   **Cross-Program Communication (CPI)**: Utilizes CPI for secure and atomic interactions between sub-programs.
*   **SPL Token Integration**: Full support for Solana Program Library (SPL) tokens for collateral and stablecoin operations.
*   **Sorted Troves (Off-Chain Architecture)**: Uses off-chain sorting with on-chain ICR validation. Client fetches all troves via RPC, sorts by ICR, and passes only neighbor hints for validation. No on-chain linked list storage, enabling unlimited scalability.
*   **Individual Collateral Ratio (ICR)**: Real-time ICR calculations are implemented across the protocol, supporting multi-collateral types and ensuring solvency checks.
*   **Redemption System**: Accepts pre-sorted trove lists from client, validates ICR ordering, supports both full and partial redemptions.

**System Design Choices:**
*   **Anchor Framework**: Used for Solana smart contract development.
*   **Rust & TypeScript**: Rust for on-chain programs and TypeScript for off-chain tests and interactions.
*   **Modular Architecture**: Separation of concerns into distinct programs (`protocol`, `oracle`, `fees`).
*   **Security Features**: Includes safe math operations, access control, input validation, atomic state consistency, PDA validation, and optimization for Solana BPF stack limits.
*   **Two-Instruction Architecture for Liquidation**: Separates data traversal from execution to optimize account ordering.
*   **Vault Signing Architecture**: All PDA vault authorities correctly sign CPIs using `invoke_signed`.
*   **BPF Stack Optimization**: Uses `UncheckedAccount` pattern to mitigate Solana BPF stack limits.

## External Dependencies

*   **Solana Blockchain**: The foundational blockchain layer.
*   **Anchor Framework**: Solana smart contract development framework.
*   **Pyth Network**: Used by the `aerospacer-oracle` program for real-time price feeds.
*   **Solana Program Library (SPL) Tokens**: Integrated for token operations within the protocol.
*   **Node.js & npm**: For running TypeScript tests and managing project dependencies.