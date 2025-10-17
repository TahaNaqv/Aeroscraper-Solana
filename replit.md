# Aerospacer Protocol - Replit Development Environment

## Overview
The Aerospacer Protocol is a decentralized lending platform (DeFi) on Solana, enabling Collateralized Debt Positions (CDPs), stablecoin (aUSD) minting, and an automated liquidation system. It integrates Pyth Network for price feeds and features a robust fee distribution mechanism. The project aims to provide a secure and efficient on-chain lending solution within the Solana ecosystem, offering a new primitive for decentralized finance.

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
*   **Dynamic Collateral Management**: Mechanisms for adding and removing collateral.
*   **Automated Liquidation System**: Ensures protocol solvency by liquidating undercollateralized positions.
*   **Stability Pool with Snapshot-Based Distribution**: Implements Liquity's Product-Sum algorithm for fair and exploit-resistant reward distribution, using P factor for pool depletion and S factors for collateral gains.
*   **Fee Distribution Mechanism**: A dual-mode system that distributes fees to either the stability pool or 50/50 to specified fee addresses, with comprehensive security validation.
*   **Oracle Integration**: Utilizes Pyth Network for real-time price feeds for all collateral assets.
*   **Cross-Program Communication (CPI)**: Extensive use of CPI for secure and atomic interactions between sub-programs.
*   **SPL Token Integration**: Full support for Solana Program Library (SPL) tokens for collateral and stablecoin operations.
*   **Sorted Troves**: Implemented as a doubly-linked list for efficient management of CDPs, supporting ICR-based positioning and auto-discovery of liquidatable troves.
*   **Individual Collateral Ratio (ICR)**: Comprehensive, real-time ICR calculations are implemented across the protocol, supporting multi-collateral types and ensuring solvency checks.
*   **Redemption System**: Integrates with the sorted troves list, supporting both full and partial redemptions.

**System Design Choices:**
*   **Anchor Framework**: Used for Solana smart contract development.
*   **Rust & TypeScript**: Rust for on-chain programs and TypeScript for off-chain tests and interactions.
*   **Comprehensive Testing**: Extensive TypeScript-based test suite covering functional, structural, and architectural aspects.
*   **Security Features**: Includes safe math operations, access control, input validation, atomic state consistency, and PDA validation to prevent forged account injections.
*   **Modular Architecture**: Separation of concerns into distinct programs (`protocol`, `oracle`, `fees`).
*   **Two-Instruction Architecture for Liquidation**: Separates data traversal from execution to optimize account ordering and adhere to Solana best practices.

## External Dependencies

*   **Solana Blockchain**: The foundational blockchain layer.
*   **Anchor Framework**: Solana smart contract development framework.
*   **Pyth Network**: Used by the `aerospacer-oracle` program for real-time price feeds.
*   **Solana Program Library (SPL) Tokens**: Integrated for token operations within the protocol.
*   **Node.js & npm**: For running TypeScript tests and managing project dependencies.

## Recent Changes

**October 17, 2025 - Compilation Errors Fixed** ✅
- **FIXED ANCHOR CONSTRAINT ERRORS**: Corrected token mint account types across all vault operations
  * Changed `UncheckedAccount<'info>` to `Account<'info, Mint>` for all mint accounts
  * Fixed `token::mint` constraints to use proper Mint account fields instead of Pubkey references
  * Added `collateral_mint: Account<'info, Mint>` to 5 instructions that needed it
  * Added Mint imports to all affected instruction files
- **Files Fixed**:
  * ✅ open_trove.rs: Fixed stable_coin_mint and added collateral_mint account
  * ✅ add_collateral.rs: Added collateral_mint account + Mint import
  * ✅ remove_collateral.rs: Added collateral_mint account + Mint import
  * ✅ borrow_loan.rs: Added collateral_mint account + Mint import
  * ✅ repay_loan.rs: Fixed stable_coin_mint + added collateral_mint account + Mint import
- **Compilation Status**:
  * ✅ aerospacer-protocol: Compiles successfully (1 deprecation warning)
  * ✅ aerospacer-oracle: Compiles successfully (6 minor warnings)
  * ✅ aerospacer-fees: Compiles successfully (3 minor warnings)
  * ✅ All programs ready for deployment

**October 16, 2025 - Protocol Vault Signing Architecture Fixed** ✅
- **FIXED CRITICAL BUG**: Implemented invoke_signed for all PDA vault authorities
  * Protocol vaults (protocol_collateral_vault, protocol_stablecoin_vault) now properly sign CPIs
  * Added `invoke_signed` with correct seeds and bumps to all vault operations
  * Vault PDAs can now self-sign for mint/transfer/burn operations
- **invoke_signed Implementations**:
  * ✅ open_trove.rs: MintTo with protocol_stablecoin_account authority + seeds
  * ✅ borrow_loan.rs: Added missing protocol_stablecoin_account + MintTo invoke_signed
  * ✅ remove_collateral.rs: Transfer with protocol_collateral_account authority + seeds
  * ✅ redeem.rs: Burn + Transfer with vault PDA authorities + seeds
  * ✅ close_trove.rs: Already had correct invoke_signed implementation
  * ✅ unstake.rs: Already had correct invoke_signed implementation
  * ✅ withdraw_liquidation_gains.rs: Already had correct invoke_signed implementation
- **Test Fixes Applied**:
  * ✅ Protocol initialization uses snake_case parameters (stable_coin_code_id, etc.)
  * ✅ All instruction parameters match Rust structs (loan_amount, collateral_amount, etc.)
  * ✅ Added init_if_needed to vault PDAs in 7 instructions
  * ✅ Added system_program to all instructions with init_if_needed
  * ✅ Stablecoin mint authority transferred to vault PDA in tests
  * ✅ Protocol is now architecturally sound for vault operations