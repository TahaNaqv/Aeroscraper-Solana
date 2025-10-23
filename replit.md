# Aerospacer Protocol - Replit Development Environment

## Overview
The Aerospacer Protocol is a decentralized lending platform on Solana. Its core purpose is to enable Collateralized Debt Positions (CDPs), mint the aUSD stablecoin, and manage an automated liquidation system. It integrates with Pyth Network for price feeds and features a robust fee distribution mechanism. The project aims to provide a secure and efficient on-chain lending solution within the Solana ecosystem, introducing a new primitive for decentralized finance.

## User Preferences
*This section will be updated as you work with the project*

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