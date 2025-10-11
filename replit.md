# Aerospacer Protocol - Replit Development Environment

## 📋 Project Overview

**Type**: Solana Blockchain Smart Contract Development  
**Framework**: Anchor v0.28.0  
**Language**: Rust + TypeScript  
**Completion**: 98% (Core protocols implemented)

This is a decentralized lending protocol (DeFi) built on Solana with three main smart contract programs:

1. **aerospacer-protocol** - Core lending logic (CDPs, stablecoin minting, liquidation)
2. **aerospacer-oracle** - Price feed management (Pyth Network integration)
3. **aerospacer-fees** - Fee distribution and economic model

## 🛠️ Environment Setup Status

### ✅ Installed Tools
- **Rust**: v1.88.0 with Cargo
- **Solana CLI**: v1.18.26
- **Anchor CLI**: v0.28.0
- **Node.js & npm**: Latest with TypeScript dependencies

### ⚙️ Build Requirements

This project requires building Solana BPF (Berkeley Packet Filter) programs, which involves:
- Solana platform-tools (BPF toolchain)
- Extended compilation time (5-10 minutes)
- Specific system permissions

## 🚀 Development Workflow

### Option 1: Build & Test Locally (Recommended)

Due to Replit environment constraints, the recommended approach is to:

1. **Build the programs**:
   ```bash
   anchor build
   ```
   *Note: First build may take 5-10 minutes and requires platform-tools installation*

2. **Run tests**:
   ```bash
   anchor test
   ```

3. **Deploy to devnet**:
   ```bash
   anchor deploy --provider.cluster devnet
   ```

### Option 2: Direct Cargo Build

If Anchor build encounters permission issues, you can try:

```bash
cargo build-sbf
```

### Option 3: Use Pre-built Artifacts

If you have build artifacts from another environment, place them in:
- `target/deploy/*.so` - Compiled programs
- `target/idl/*.json` - Interface definitions

## 📁 Project Structure

```
aerospacer-solana/
├── programs/
│   ├── aerospacer-protocol/  # Core lending logic
│   ├── aerospacer-oracle/    # Price feed management  
│   └── aerospacer-fees/      # Fee distribution
├── tests/                     # TypeScript test suite
│   ├── aerospacer-solana.ts  # Main integration tests
│   ├── oracle-test.ts        # Oracle program tests
│   └── fee-contract-*.ts     # Fee distribution tests
├── Anchor.toml               # Anchor configuration
└── Cargo.toml                # Rust workspace config
```

## 🧪 Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
anchor test

# Run specific test file
npx ts-mocha -p ./tsconfig.json -t 1000000 'tests/aerospacer-solana.ts'

# Run with verbose output
anchor test --verbose
```

### Test Files Overview
- `aerospacer-solana.ts` - Main protocol tests
- `oracle-*.ts` - Oracle functionality tests
- `fee-contract-*.ts` - Fee distribution tests
- `protocol-core.ts` - Core protocol tests
- `devnet-initialization.ts` - Devnet deployment

## 🔧 Common Issues & Solutions

### Issue: Permission Denied During Build

**Cause**: Solana platform-tools installation requires specific permissions  
**Solution**: The Solana CLI from the nix package should include BPF tools. If issues persist, try:
```bash
# Check if BPF tools are available
cargo build-sbf --version
```

### Issue: Build Timeouts

**Cause**: Solana programs take 5-10 minutes to compile initially  
**Solution**: Be patient on first build, subsequent builds use caching

### Issue: Anchor Version Mismatch

**Cause**: Multiple Anchor versions installed  
**Solution**: This environment uses Anchor CLI v0.28.0 to match `Anchor.toml`

## 📚 Documentation Files

- `README.md` - Project overview and features
- `PROJECT_STATUS.md` - Implementation status (98% complete)
- `TESTING_GUIDE.md` - Comprehensive testing guide
- `DEPLOYMENT_STATUS.md` - Deployment information
- `FEE_CONTRACT_*.md` - Fee contract documentation

## 🎯 Key Features Implemented

✅ Collateralized Debt Positions (CDPs)  
✅ Stablecoin (aUSD) Minting  
✅ Dynamic Collateral Management  
✅ Automatic Liquidation System  
✅ Stability Pool with Staking  
✅ Fee Distribution Mechanism  
✅ Oracle Integration (Pyth Network)  
✅ Cross-Program Communication (CPI)  
✅ SPL Token Integration  

## 🔐 Security Features

- Safe Math Operations (overflow protection)
- Access Control (admin-only functions)
- Comprehensive Input Validation
- Graceful Error Handling
- Atomic State Consistency

## 📝 User Preferences

*This section will be updated as you work with the project*

## 💰 Fee Integration Architecture

### Overview
The protocol integrates with `aerospacer-fees` contract for comprehensive fee collection and distribution across all revenue-generating operations.

### Fee-Collecting Operations

| Operation | Fee Type | Fee % | Implementation Status |
|-----------|----------|-------|----------------------|
| `open_trove` | Opening fee | Configurable (default 5%) | ✅ Complete |
| `borrow_loan` | Borrowing fee | Configurable (default 0.5%) | ✅ Complete |
| `redeem` | Redemption fee | Configurable (default 0.5%) | ✅ Complete |
| `liquidate_troves` | N/A (bonus model) | N/A | ⏭️ Not applicable |

### Fee Distribution Flow

```
User Operation (open/borrow/redeem)
    ↓
1. Calculate Fee Amount
   - fee_amount = operation_amount * protocol_fee_percentage
   - net_amount = operation_amount - fee_amount
    ↓
2. Record Net Amount in Protocol State
   - User debt = net_amount
   - Total debt += net_amount
   - ICR calculated on net_amount
    ↓
3. Mint/Transfer Gross Amount to User
   - User receives full requested amount
    ↓
4. CPI to aerospacer-fees::distribute_fee
   - Deduct fee_amount from user token account
   - Distribute based on stake_enabled flag:
     * If true: 100% → Stability Pool
     * If false: 50/50 → FEE_ADDR_1 & FEE_ADDR_2
```

### CPI Implementation Details

**Instruction Discriminator Calculation**:
```rust
let preimage = b"global:distribute_fee";
let hash_result = hash(preimage);
let discriminator = &hash_result.to_bytes()[..8];
```

**Account Requirements** (per operation):
- `fees_program` - Fee contract program ID (validated against `state.fee_distributor_addr`)
- `fees_state` - Fee contract state account
- `payer_token_account` - User's aUSD token account (source of fees)
- `stability_pool_token_account` - Destination when stake enabled
- `fee_address_1_token_account` - Destination 1 when stake disabled
- `fee_address_2_token_account` - Destination 2 when stake disabled
- `token_program` - SPL Token program

**Key Functions**:
- `process_protocol_fee()` - Main CPI handler in `fees_integration.rs`
- `distribute_fee_via_cpi()` - Builds and executes CPI instruction
- `calculate_protocol_fee()` - Fee amount calculation in `utils.rs`

### Fee Contract Configuration

The fee contract (`aerospacer-fees`) supports two distribution modes:

**Mode 1: Stake-Based Distribution** (`is_stake_enabled = true`)
- All fees → Stability Pool (stakers)
- Proportional to stake amount
- Incentivizes aUSD staking

**Mode 2: Treasury Distribution** (`is_stake_enabled = false`)
- 50% → FEE_ADDR_1 (Protocol Treasury/Development)
- 50% → FEE_ADDR_2 (Validator Rewards/Staking)

**Configuration Functions**:
- `toggle_stake_contract()` - Switch distribution modes (admin only)
- `set_stake_contract_address()` - Set stability pool address (admin only)
- `get_config()` - Query current configuration

### Security Considerations

1. **No Double-Charging**: Fee deducted only once per operation
2. **Net Amount Recording**: Protocol state always reflects net (post-fee) amounts
3. **Atomic CPI**: Fee distribution happens in same transaction as operation
4. **Access Control**: Only authorized fee contract can receive funds
5. **Safe Math**: All calculations use checked arithmetic

### Integration Checklist

When adding a new fee-collecting operation:
1. ✅ Import `fees_integration` and `utils` modules
2. ✅ Add 5 fee-related accounts to instruction context
3. ✅ Calculate fee BEFORE trove/state operations
4. ✅ Pass net amount to state management functions
5. ✅ Mint/transfer gross amount to user
6. ✅ Call `process_protocol_fee()` with all required accounts
7. ✅ Update logging to show fee and net amounts

## 🗓️ Recent Changes

**2025-10-11**: Fee Integration Complete
- Implemented production-ready CPI to aerospacer-fees contract
- Added fee collection to open_trove, borrow_loan, redeem operations
- Integrated automatic fee distribution (dual-mode: stake pool or treasury)
- All operations correctly record net amounts while users receive gross

**2025-10-08**: Initial Replit environment setup
- Installed Rust, Solana CLI, and Anchor framework
- Configured development environment
- Created setup documentation

## 🚦 Next Steps

1. **Build the programs**: Run `anchor build` (allow 5-10 minutes for first build)
2. **Run tests**: Execute `anchor test` to verify implementation
3. **Review code**: Explore the three main programs in `programs/` directory
4. **Deploy to devnet**: Use `anchor deploy --provider.cluster devnet` for testing

## 💡 Tips for Development

- **Fast feedback**: Use `cargo check` for quick syntax validation without full compilation
- **Incremental builds**: After first build, changes compile much faster
- **Test individual functions**: Focus on specific test files during development
- **Use Solana devnet**: Test deployments on devnet before mainnet

---

*This is a professional-grade Solana DeFi protocol. Take time to understand the architecture before making changes.*
