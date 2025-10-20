# Aerospacer Oracle - Devnet Testing Guide

Complete guide for testing the Aerospacer Oracle contract on Solana devnet with real Pyth Network price feed integration.

---

## 📋 Prerequisites

Before starting, ensure you have:

1. **Solana CLI** installed and configured
2. **Anchor CLI** v0.31.1 installed
3. **Node.js** and npm installed
4. **Devnet SOL** in your wallet (minimum 5 SOL)

---

## 🚀 Quick Start (5 Steps)

### Step 1: Get Devnet SOL

```bash
# Configure Solana CLI for devnet
solana config set --url https://api.devnet.solana.com

# Check your wallet address
solana address

# Request devnet SOL (can repeat multiple times)
solana airdrop 2

# Verify balance
solana balance
```

### Step 2: Build the Oracle Contract

```bash
# Build all programs (protocol, oracle, fees)
anchor build

# This generates:
# - target/deploy/*.so (program binaries)
# - target/types/*.ts (TypeScript types)
# - target/idl/*.json (IDL files)
```

### Step 3: Deploy to Devnet

```bash
# Deploy all programs to devnet
anchor deploy --provider.cluster devnet

# Expected output:
# Program Id: 8zG12srZdYaJPjWzCAJhwyxF7wWTz5spbmehxWpV5Q9M (oracle)
# Deploy success
```

### Step 4: Initialize Oracle & Configure Assets

```bash
# Initialize oracle contract on devnet
npm run init_oracle_devnet

# This creates:
# - New oracle state account
# - Saves config to scripts/.oracle-devnet-config.json
# ⚠️  SAVE THE STATE ACCOUNT ADDRESS that prints out!

# Add SOL, ETH, BTC with Pyth price feeds
npm run add_assets_devnet

# Expected output:
# ✅ SOL configured successfully
# ✅ ETH configured successfully
# ✅ BTC configured successfully
```

### Step 5: Test Pyth Integration

```bash
# Query real-time prices from Pyth devnet
npm run test_prices_devnet

# Expected output:
# SOL/USD: $183.41 ± $0.12
# ETH/USD: $7,891.58 ± $10.50
# BTC/USD: $125,000.00 ± $150.00

# Run comprehensive test suite (66 tests)
npm run test-oracle-devnet
```

---

## ⚠️ Important: TypeScript Client Fix

### Issue with `.view()` Method

The oracle contract returns custom `PriceResponse` structs, but Anchor's `.view()` method **cannot deserialize custom struct return types**. This causes empty error messages when querying prices.

**❌ This DOESN'T Work:**
```typescript
const price = await program.methods
  .getPrice({ denom: "SOL" })
  .accounts({ ... })
  .view();  // ❌ Fails with empty error
```

### Solution: Use Simulation + Log Parsing

**✅ This WORKS:**
```typescript
// Build instruction
const ix = await program.methods
  .getPrice({ denom: "SOL" })
  .accounts({
    state: stateAccountPubkey,
    pythPriceAccount: SOL_PRICE_FEED,
    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
  })
  .instruction();

// Simulate transaction
const tx = new anchor.web3.Transaction().add(ix);
const simulation = await provider.connection.simulateTransaction(
  tx,
  [provider.wallet.payer],
  false
);

// Parse price data from program logs
const logs = simulation.value.logs || [];
// logs contain: "Program log: Price: 183415750000 ± 1000 x 10^-9"
```

### Why This Works

1. The Rust contract logs all price data via `msg!()` statements
2. Simulation returns these logs without executing on-chain
3. We parse the logs to extract the `PriceResponse` data

### For Protocol Integration (CPI)

**✅ CPI Works Perfectly** - Cross-program invocation (CPI) from the protocol contract works correctly because Solana CPI **does support returning custom structs**. The TypeScript client limitation does NOT affect protocol → oracle CPI calls.

```rust
// Protocol can call oracle via CPI and get PriceResponse
let price_response: PriceResponse = get_price_cpi(
    oracle_program,
    oracle_state,
    pyth_account,
    denom
)?;
```

---

## 📊 What Gets Tested

### Test Suite Overview (66 Total Tests)

| Test File | Tests | Coverage |
|-----------|-------|----------|
| **oracle-initialization.ts** | 6 | Contract setup, get_config validation |
| **oracle-admin-controls.ts** | 10 | set_data, remove_data, batch operations, authorization |
| **oracle-price-queries.ts** | 12 | get_price, get_all_prices with **real Pyth integration** |
| **oracle-info-queries.ts** | 8 | check_denom, get_all_denoms, get_price_id |
| **oracle-security.ts** | 10 | Authorization, validation, attack prevention |
| **oracle-edge-cases.ts** | 12 | Batch limits, edge cases, rapid operations |
| **oracle-integration.ts** | 8 | Protocol CPI integration, liquidation simulation |

### Instructions Tested (12/12 = 100%)

- ✅ `initialize` - Oracle setup with Pyth provider
- ✅ `update_oracle_address` - Change Pyth provider
- ✅ `set_data` - Configure single asset
- ✅ `set_data_batch` - Configure multiple assets (up to 100)
- ✅ `remove_data` - Remove asset support
- ✅ `get_price` - Query real-time price from Pyth
- ✅ `get_all_prices` - Batch price query
- ✅ `update_pyth_price` - Refresh price feed
- ✅ `get_config` - Oracle configuration
- ✅ `get_all_denoms` - List supported assets
- ✅ `get_price_id` - Get Pyth price feed ID
- ✅ `check_denom` - Check asset support

---

## 🔍 Pyth Network Integration Details

### Devnet Price Feeds Configured

```typescript
SOL/USD:
  Address: J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix
  Price ID: ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d
  Decimal: 9

ETH/USD:
  Address: EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw
  Price ID: ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace
  Decimal: 18

BTC/USD:
  Address: HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J
  Price ID: e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43
  Decimal: 8
```

### Price Update Frequency

- **Pyth devnet updates**: Every ~400ms
- **Staleness threshold**: 60 seconds (hardcoded)
- **Minimum confidence**: 1000 (hardcoded)

---

## 🛠️ Available npm Scripts

```bash
# Deployment
npm run deploy_devnet              # Deploy all programs to devnet

# Oracle Setup
npm run init_oracle_devnet         # Initialize oracle contract
npm run add_assets_devnet          # Add SOL/ETH/BTC with Pyth feeds
npm run test_prices_devnet         # Quick price query test

# Testing
npm run test-oracle-devnet         # Run all oracle tests (66 tests)
npm run test-fee-devnet           # Run all fee tests (67 tests)
```

---

## 📝 Important Files

### Created During Setup

```
scripts/.oracle-devnet-config.json    # Oracle state account & config
                                      # ⚠️  DO NOT DELETE - needed by scripts
```

### Script Files

```
scripts/init-oracle-devnet.ts         # Initialize oracle
scripts/add-assets-devnet.ts          # Configure assets
scripts/test-prices-devnet.ts         # Query prices
```

### Test Files

```
tests/oracle-initialization.ts        # Setup tests
tests/oracle-admin-controls.ts        # Admin operations
tests/oracle-price-queries.ts         # Pyth integration
tests/oracle-info-queries.ts          # Info queries
tests/oracle-security.ts              # Security & auth
tests/oracle-edge-cases.ts            # Edge cases
tests/oracle-integration.ts           # Protocol CPI
```

---

## 🐛 Troubleshooting

### Issue: "Insufficient funds"

```bash
# Request more devnet SOL
solana airdrop 2

# Check balance
solana balance

# You need ~5 SOL for deployment and testing
```

### Issue: "State account not found"

```bash
# Re-run initialization
npm run init_oracle_devnet

# Make sure to save the state account address it prints
```

### Issue: "Price too old" error

This means Pyth price is stale (>60 seconds old). On devnet, Pyth updates every ~400ms, so this usually means:
- Pyth devnet is experiencing issues
- Check https://pyth.network for devnet status

### Issue: Tests timing out

```bash
# Increase timeout in test command (already set to 1000000ms)
# Or run specific test file:
npx ts-mocha -p ./tsconfig.json -t 1000000 tests/oracle-price-queries.ts
```

### Issue: "Invalid price ID" error

- Ensure price IDs are exactly 64 hexadecimal characters
- No "0x" prefix
- Lowercase hex only

---

## 🔐 Security Validations Tested

1. ✅ **Admin-only operations** - Only admin can modify oracle state
2. ✅ **Price ID format** - Must be 64-char hex string
3. ✅ **Token account ownership** - Account owner validation
4. ✅ **Batch size limits** - Maximum 100 assets per batch
5. ✅ **Empty data rejection** - No empty denoms or price IDs
6. ✅ **Pyth account validation** - Correct Pyth price feed required
7. ✅ **Staleness checks** - Prices must be <60 seconds old
8. ✅ **Confidence validation** - Minimum confidence threshold enforced

---

## 🎯 Next Steps

After successful devnet testing:

1. **Review test results** - Ensure all 66 tests pass
2. **Test protocol integration** - Connect your lending protocol to oracle
3. **Mainnet preparation** - Update to mainnet Pyth feeds
4. **Audit** - Security audit before mainnet deployment

---

## 📚 Additional Resources

- **Pyth Network Docs**: https://docs.pyth.network/price-feeds/use-real-time-data/solana
- **Pyth Price Feed IDs**: https://pyth.network/developers/price-feed-ids
- **Solana Devnet**: https://api.devnet.solana.com
- **Anchor Docs**: https://www.anchor-lang.com

---

## ✅ Success Checklist

- [ ] Devnet SOL obtained (5+ SOL)
- [ ] Programs deployed to devnet
- [ ] Oracle initialized successfully
- [ ] Assets configured (SOL, ETH, BTC)
- [ ] Real-time prices queried from Pyth
- [ ] All 66 tests passing
- [ ] State account address saved

**Once complete, your oracle is ready for protocol integration!** 🎉

---

## 📞 Support

If you encounter issues:

1. Check this guide's troubleshooting section
2. Verify devnet status: https://status.solana.com
3. Check Pyth devnet status: https://pyth.network
4. Review test logs for specific error messages

**Happy testing!** 🚀
