# Aerospacer Protocol - Replit Development Environment

### Overview
The Aerospacer Protocol is a decentralized lending platform (DeFi) on Solana, enabling Collateralized Debt Positions (CDPs), stablecoin (aUSD) minting, and an automated liquidation system. It integrates Pyth Network for price feeds and features a robust fee distribution mechanism. The project aims to provide a secure and efficient on-chain lending solution within the Solana ecosystem, offering a new primitive for decentralized finance.

### Recent Changes
**October 12, 2025 - Stability Pool Snapshot Distribution (Liquity Product-Sum Algorithm)**
- Implemented snapshot-based stability pool distribution using Liquity's Product-Sum algorithm to prevent economic exploits
- Added P factor (product/depletion) tracking to StateAccount for fair reward distribution during pool depletion
- Created StabilityPoolSnapshot PDAs to track S factors (sum/gain) per collateral denomination
- Updated UserStakeAmount with p_snapshot, epoch_snapshot for compounded stake calculations
- Created UserCollateralSnapshot PDAs for lazy S snapshot initialization per user per collateral type
- Implemented helper functions: calculate_compounded_stake() and calculate_collateral_gain()
- Updated stake/unstake/withdraw_liquidation_gains instructions to use snapshot calculations
- Added epoch rollover logic when P factor drops below 10^9
- **Security:** Prevents deposit/withdraw gaming, frontrunning attacks, and ensures mathematically fair proportional rewards

### User Preferences
*This section will be updated as you work with the project*

### System Architecture

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
*   **Stability Pool with Snapshot-Based Distribution**: Implements Liquity's Product-Sum algorithm for fair and exploit-resistant reward distribution. Users stake aUSD to earn liquidation gains proportionally based on snapshots (P factor tracks pool depletion, S factors track collateral gains per denomination). Prevents economic gaming attacks and ensures fair proportional rewards.
*   **Fee Distribution Mechanism**: A flexible system to collect and distribute protocol fees.
*   **Oracle Integration**: Utilizes Pyth Network for real-time price feeds for all collateral assets.
*   **Cross-Program Communication (CPI)**: Extensive use of CPI for secure and atomic interactions between sub-programs.
*   **SPL Token Integration**: Full support for Solana Program Library (SPL) tokens for collateral and stablecoin operations.
*   **Sorted Troves**: Implemented as a doubly-linked list for efficient management of CDPs, supporting ICR-based positioning (riskiest to safest) and auto-discovery of liquidatable troves.
*   **Individual Collateral Ratio (ICR)**: Comprehensive, real-time ICR calculations are implemented across the protocol, supporting multi-collateral types and ensuring solvency checks.
*   **Redemption System**: Redesigned to integrate with the sorted troves list, supporting both full and partial redemptions with robust state management and cleanup.

**System Design Choices:**
*   **Anchor Framework**: Used for Solana smart contract development.
*   **Rust & TypeScript**: Rust for on-chain programs and TypeScript for off-chain tests and interactions.
*   **Comprehensive Testing**: Extensive TypeScript-based test suite.
*   **Security Features**: Includes safe math operations, access control, input validation, atomic state consistency, and PDA validation to prevent forged account injections.
*   **Modular Architecture**: Separation of concerns into distinct programs (`protocol`, `oracle`, `fees`).
*   **Two-Instruction Architecture for Liquidation**: Separates data traversal from execution to optimize account ordering and adhere to Solana best practices.

### External Dependencies

*   **Solana Blockchain**: The foundational blockchain layer.
*   **Anchor Framework**: Solana smart contract development framework.
*   **Pyth Network**: Used by the `aerospacer-oracle` program for real-time price feeds.
*   **Solana Program Library (SPL) Tokens**: Integrated for token operations within the protocol.
*   **Node.js & npm**: For running TypeScript tests and managing project dependencies.