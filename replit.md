# Aerospacer Protocol - Replit Development Environment

## Overview
The Aerospacer Protocol is a decentralized lending platform on Solana that enables Collateralized Debt Positions (CDPs), minting of the aUSD stablecoin, and an automated liquidation system. It integrates with Pyth Network for price feeds and includes a robust fee distribution mechanism. The project's primary goal is to deliver a secure and efficient on-chain lending solution within the Solana ecosystem, introducing a new primitive for decentralized finance.

## User Preferences
*This section will be updated as you work with the project*

## System Architecture

**Core Programs:**
The project is built with Anchor v0.28.0 in Rust and comprises three primary Solana smart contract programs:
1.  **aerospacer-protocol**: Manages core lending logic, including CDPs, stablecoin minting, and liquidation.
2.  **aerospacer-oracle**: Handles price feed management, primarily integrating with the Pyth Network.
3.  **aerospacer-fees**: Manages fee collection and distribution.

**UI/UX Decisions:**
The design emphasizes transparent and auditable on-chain interactions, ensuring all state changes and operations are publicly verifiable on the Solana blockchain.

**Technical Implementations & Feature Specifications:**
*   **Collateralized Debt Positions (CDPs)**: Users can lock collateral to mint aUSD stablecoins.
*   **Stablecoin (aUSD) Minting**: Supports the minting of its native stablecoin, aUSD.
*   **Automated Liquidation System**: Ensures protocol solvency by liquidating undercollateralized positions.
*   **Stability Pool with Snapshot-Based Distribution**: Implements Liquity's Product-Sum algorithm for fair reward distribution.
*   **Fee Distribution Mechanism**: A dual-mode system that distributes fees to either the stability pool or splits them 50/50 to specified fee addresses.
*   **Oracle Integration**: Utilizes Pyth Network for real-time price feeds for all collateral assets, with dynamic collateral discovery via CPI.
*   **Cross-Program Communication (CPI)**: Extensive use of CPI for secure and atomic interactions between sub-programs.
*   **SPL Token Integration**: Full support for Solana Program Library (SPL) tokens for collateral and stablecoin operations.
*   **Sorted Troves**: Implemented as a doubly-linked list for efficient management of CDPs, supporting ICR-based positioning and auto-discovery of liquidatable troves.
*   **Individual Collateral Ratio (ICR)**: Comprehensive, real-time ICR calculations are implemented across the protocol, supporting multi-collateral types and ensuring solvency checks.
*   **Redemption System**: Integrates with the sorted troves list, supporting both full and partial redemptions.
*   **Admin State Recovery**: Includes a `reset_sorted_troves` instruction for recovering from corrupted sorted troves state on devnet, and a `close_node` instruction for emergency cleanup of individual user Node accounts.

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

## Recent Fixes & Devnet Testing Notes

### Token Account Creation Fix (October 22, 2025)
Fixed "Provided owner is not allowed" errors in test suite by adding existence checks before creating user1 and user2 stablecoin accounts. This prevents the Associated Token Program from rejecting duplicate account creation attempts when tests are run multiple times on devnet where accounts persist between runs.

### Node Account Cleanup for Devnet Testing
**Issue**: When running tests multiple times on devnet, Node PDA accounts from previous test runs persist and cause "Allocate: account already in use" errors during `open_trove` instruction execution.

**Solution**: Created `scripts/close-specific-nodes-devnet.ts` diagnostic script that:
1. Targets the exact Node PDA addresses from error logs
2. Fetches and decodes Node data to identify the user public key
3. Verifies PDA derivation matches expected pattern `[b"node", user_pubkey]`
4. Calls `close_node` instruction to properly cleanup accounts

**Important**: The `close-user-nodes-devnet.ts` script derives Node PDAs from hardcoded user public keys, but may not match actual devnet state if different keypairs were used in previous test runs. Always use `close-specific-nodes-devnet.ts` with actual failing PDA addresses from error logs for reliable cleanup.