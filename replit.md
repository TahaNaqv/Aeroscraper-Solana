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

**October 17, 2025 - Production Readiness Assessment Completed** ✅
- **COMPREHENSIVE PROTOCOL REVIEW**: Analyzed all 13 instruction handlers, 8 core modules, and integration architecture
  * ✅ Reviewed: initialize, open_trove, add_collateral, remove_collateral, borrow_loan, repay_loan, close_trove, liquidate_troves, query_liquidatable_troves, stake, unstake, withdraw_liquidation_gains, redeem
  * ✅ Validated: Liquity Product-Sum algorithm (P/S factor calculations correct)
  * ✅ Confirmed: Vault signing architecture (invoke_signed patterns correct)
  * ✅ Verified: Safe math operations, access control, state consistency
- **HONEST STATUS ASSESSMENT**:
  * ⚠️ **Current Phase**: Integration-testing (NOT production-ready)
  * ⚠️ **Critical Blockers**: Oracle prices hardcoded (utils/mod.rs), redemption system mocked (trove_management.rs), liquidation gain distribution incomplete
  * ⚠️ **Production Readiness Score**: 5.9/10
  * ⚠️ **Recommendation**: DO NOT DEPLOY until blockers resolved
- **DOCUMENTATION CREATED**:
  * ✅ PRODUCTION_READINESS_REPORT.md: Comprehensive 400+ line assessment with specific code examples, blocker analysis, and 2-4 week roadmap to production
  * ✅ Transparent evaluation: Separates "What Works" vs "What's Missing"
  * ✅ Actionable next steps: Oracle CPI implementation, redemption system completion, integration testing

**October 17, 2025 - Compilation Errors Fixed** ✅
- **FIXED ANCHOR CONSTRAINT ERRORS**: Corrected token mint account types across all vault operations
- **Compilation Status**:
  * ✅ aerospacer-protocol: Compiles successfully (1 deprecation warning)
  * ✅ aerospacer-oracle: Compiles successfully (6 minor warnings)
  * ✅ aerospacer-fees: Compiles successfully (3 minor warnings)

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