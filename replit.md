# Aerospacer Protocol - Replit Development Environment

## Overview
The Aerospacer Protocol is a decentralized lending platform on Solana. Its core purpose is to enable Collateralized Debt Positions (CDPs), mint the aUSD stablecoin, and manage an automated liquidation system. It integrates with Pyth Network for price feeds and features a robust fee distribution mechanism. The project aims to provide a secure and efficient on-chain lending solution within the Solana ecosystem, introducing a new primitive for decentralized finance.

## User Preferences
*This section will be updated as you work with the project*

## Recent Changes (2025-01-23)

### Protocol Fee Integration Tests Fixed
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
*   **Sorted Troves**: Implemented as a doubly-linked list for efficient CDP management, supporting ICR-based positioning and auto-discovery of liquidatable troves.
*   **Individual Collateral Ratio (ICR)**: Real-time ICR calculations are implemented across the protocol, supporting multi-collateral types and ensuring solvency checks.
*   **Redemption System**: Integrates with the sorted troves list, supporting both full and partial redemptions.
*   **Admin State Recovery**: Includes instructions for recovering from corrupted sorted troves state on devnet and emergency cleanup of individual user Node accounts.

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