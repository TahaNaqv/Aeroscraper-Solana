# Aerospacer Protocol - Devnet Testing Guide

## 🚀 **Testing Status**

**Current Status**: ✅ **Programs Deployed & Ready for Testing**  
**Network**: Solana Devnet  
**Programs**: 4/4 Deployed Successfully

---

## 📊 **Deployed Programs**

| Program | Program ID | Status |
|---------|------------|---------|
| **aerospacer_oracle** | `8zG12srZdYaJPjWzCAJhwyxF7wWTz5spbmehxWpV5Q9M` | ✅ Live |
| **aerospacer_protocol** | `9sk8X11GWtZjzXWfkcLMRD6tmuhmiBKgMXsmx9bEh5YQ` | ✅ Live |
| **aerospacer_fees** | `AHmGKukQky3mDHLmFyJYcEaFub69vp2QqeSW7EbVpJjZ` | ✅ Live |
| **aerospacer_solana** | `6kJZg8PDkutRui282AnspEnLcyExxcpsbCvyfBoTcDwN` | ✅ Live |

---

## 🧪 **Testing Approach**

### **Phase 1: Basic Functionality Testing** ✅ **READY**

#### **1. Oracle Program Testing**
```bash
# Initialize Oracle Program
anchor test --provider.cluster devnet --grep "Should initialize oracle program"

# Set Collateral Data
anchor test --provider.cluster devnet --grep "Should set collateral data"
```

#### **2. Fees Program Testing**
```bash
# Initialize Fees Program
anchor test --provider.cluster devnet --grep "Should initialize fees program"
```

#### **3. Protocol Program Testing**
```bash
# Initialize Protocol Program
anchor test --provider.cluster devnet --grep "Should initialize protocol program"

# Test Core Operations
anchor test --provider.cluster devnet --grep "Should open a trove"
anchor test --provider.cluster devnet --grep "Should add collateral to trove"
anchor test --provider.cluster devnet --grep "Should borrow loan"
anchor test --provider.cluster devnet --grep "Should stake stablecoins"
```

### **Phase 2: Integration Testing** 🔄 **IN PROGRESS**

#### **1. Cross-Program Communication**
- Test Oracle ↔ Protocol communication
- Test Fees ↔ Protocol communication
- Test token transfers between programs

#### **2. Error Handling**
- Test insufficient collateral scenarios
- Test invalid parameter handling
- Test unauthorized access attempts

#### **3. Edge Cases**
- Test maximum values
- Test minimum values
- Test boundary conditions

---

## 🔧 **Manual Testing Commands**

### **1. Verify Program Deployment**
```bash
# Check Oracle Program
solana program show 8zG12srZdYaJPjWzCAJhwyxF7wWTz5spbmehxWpV5Q9M

# Check Protocol Program
solana program show 9sk8X11GWtZjzXWfkcLMRD6tmuhmiBKgMXsmx9bEh5YQ

# Check Fees Program
solana program show AHmGKukQky3mDHLmFyJYcEaFub69vp2QqeSW7EbVpJjZ
```

### **2. Check Account Balances**
```bash
# Check SOL balance
solana balance

# Check token balances (after setup)
spl-token balance <TOKEN_MINT_ADDRESS>
```

### **3. Run Individual Tests**
```bash
# Run only Oracle tests
anchor test --provider.cluster devnet --grep "Oracle"

# Run only Protocol tests
anchor test --provider.cluster devnet --grep "Protocol"

# Run only initialization tests
anchor test --provider.cluster devnet --grep "initialize"
```

---

## 📋 **Testing Checklist**

### **Pre-Testing Setup** ✅ **COMPLETE**
- [x] Programs deployed to devnet
- [x] Program IDs configured in Anchor.toml
- [x] Admin account has sufficient SOL
- [x] Token accounts created
- [x] Test environment configured

### **Basic Functionality Tests** 🔄 **IN PROGRESS**
- [ ] Oracle program initialization
- [ ] Oracle data setting
- [ ] Fees program initialization
- [ ] Protocol program initialization
- [ ] Trove opening
- [ ] Collateral addition
- [ ] Loan borrowing
- [ ] Stablecoin staking

### **Integration Tests** ⏳ **PENDING**
- [ ] Cross-program communication
- [ ] Token transfer validation
- [ ] Error handling scenarios
- [ ] Edge case testing
- [ ] Performance testing

### **Advanced Tests** ⏳ **PENDING**
- [ ] Liquidation system
- [ ] Fee distribution
- [ ] Oracle price feeds
- [ ] Multi-user scenarios
- [ ] Stress testing

---

## 🚨 **Known Issues & Workarounds**

### **1. Devnet Airdrop Rate Limits**
- **Issue**: Cannot airdrop SOL due to rate limits
- **Solution**: ✅ Use existing SOL balance (3.33 SOL available)
- **Status**: Resolved

### **2. Token Account Creation**
- **Issue**: Duplicate account creation errors
- **Solution**: ✅ Use same accounts for user and protocol operations
- **Status**: Resolved

### **3. Test Setup Complexity**
- **Issue**: Complex multi-account setup
- **Solution**: ✅ Simplified to use admin account for all operations
- **Status**: Resolved

---

## 🎯 **Success Criteria**

### **Phase 1 Success** ✅ **ACHIEVED**
- [x] All programs deployed successfully
- [x] Basic setup working
- [x] Token accounts created
- [x] No critical errors in deployment

### **Phase 2 Success** 🎯 **TARGET**
- [ ] All basic functionality tests pass
- [ ] Cross-program communication working
- [ ] Error handling working correctly
- [ ] Core lending operations functional

### **Phase 3 Success** 🎯 **TARGET**
- [ ] All integration tests pass
- [ ] Performance acceptable
- [ ] Security validated
- [ ] Ready for mainnet preparation

---

## 🚀 **Next Actions**

### **Immediate (Next 24 hours)**
1. **Run Basic Tests**: Execute individual test cases
2. **Debug Issues**: Fix any remaining test failures
3. **Document Results**: Record test outcomes

### **Short-term (Next 1-2 weeks)**
1. **Complete Integration Testing**: Test all cross-program interactions
2. **Performance Optimization**: Optimize gas usage and performance
3. **Security Review**: Conduct security analysis

### **Medium-term (Next 1-2 months)**
1. **Community Testing**: Invite community to test
2. **Frontend Development**: Build user interface
3. **Mainnet Preparation**: Prepare for production deployment

---

## 🔗 **Oracle CPI Integration Testing**

### **Overview**
The protocol now uses **real Cross-Program Invocation (CPI)** to the `aerospacer-oracle` contract for price feeds. All protocol instructions that require price data make CPI calls to `get_price` with Pyth Network integration.

### **CPI Integration Architecture**

#### **Affected Instructions** (All 6 operations)
1. `open_trove` - Validates ICR using oracle price
2. `add_collateral` - Recalculates ICR with new collateral
3. `remove_collateral` - Ensures ICR remains above MCR
4. `borrow_loan` - Validates ICR after additional borrowing
5. `repay_loan` - Updates ICR after debt reduction
6. `liquidate_troves` - Gets current prices for liquidation

#### **Required Accounts for CPI**
Each instruction now requires **4 oracle-related accounts**:
```rust
// In instruction context
pub oracle_program: Program<'info, AerospacerOracle>,
pub oracle_state: Account<'info, OracleStateAccount>,
pub pyth_price_account: AccountInfo<'info>,  // NEW: Pyth price feed
pub clock: Sysvar<'info, Clock>,              // NEW: For staleness check
```

### **Test Setup Requirements**

#### **1. Pyth Price Feed Addresses** (Devnet)
```typescript
import { SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";

// Real Pyth Network price feeds (Devnet)
const PYTH_PRICE_FEEDS = {
  SOL: new PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"),  // SOL/USD
  ETH: new PublicKey("EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw"),  // ETH/USD
  BTC: new PublicKey("HovQMDrbAgAYPCmHVSrezcSmKQtUUJtXHimcwhYWrz8z"),  // BTC/USD
};
```

#### **2. Updated Test Pattern** (Example: `open_trove`)
```typescript
// OLD (Missing CPI accounts)
await protocolProgram.methods
  .openTrove({
    collateralAmount: new anchor.BN(collateralAmount),
    loanAmount: new anchor.BN(loanAmount),
    collateralDenom: "SOL",
  })
  .accounts({
    trove: user1Trove,
    state: protocolState,
    user: user1.publicKey,
    userCollateralAccount: user1CollateralAccount,
    userStablecoinAccount: user1StablecoinAccount,
    stablecoinMint: stablecoinMint,
    oracleProgram: oracleProgram.programId,
    oracleState: oracleState,
    feesProgram: feesProgram.programId,
    feesState: feesState,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .signers([user1])
  .rpc();

// NEW (With CPI accounts)
await protocolProgram.methods
  .openTrove({
    collateralAmount: new anchor.BN(collateralAmount),
    loanAmount: new anchor.BN(loanAmount),
    collateralDenom: "SOL",
  })
  .accounts({
    trove: user1Trove,
    state: protocolState,
    user: user1.publicKey,
    userCollateralAccount: user1CollateralAccount,
    userStablecoinAccount: user1StablecoinAccount,
    stablecoinMint: stablecoinMint,
    oracleProgram: oracleProgram.programId,
    oracleState: oracleState,
    pythPriceAccount: PYTH_PRICE_FEEDS.SOL,      // NEW
    clock: SYSVAR_CLOCK_PUBKEY,                   // NEW
    feesProgram: feesProgram.programId,
    feesState: feesState,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .signers([user1])
  .rpc();
```

### **Test Files Requiring Updates**

| Test File | Instructions to Update | Status |
|-----------|------------------------|--------|
| `tests/protocol-core.ts` | open_trove, add_collateral, borrow_loan, repay_loan, remove_collateral | ⏳ Needs Update |
| `tests/basic-protocol-test.ts` | open_trove, add_collateral | ⏳ Needs Update |
| `tests/deep-protocol-test.ts` | All 6 operations | ⏳ Needs Update |
| `tests/aerospacer-solana.ts` | open_trove, liquidate_troves | ⏳ Needs Update |

### **CPI Error Handling Tests**

#### **Scenarios to Test**

1. **Stale Price Data** (>5 minutes old)
   ```typescript
   // Test with old Pyth price (should fail staleness check)
   it("Should reject operation with stale price (>5 min)", async () => {
     // Use a Pyth account with timestamp > 300 seconds old
     // Expected error: "Invalid amount" (from staleness check)
   });
   ```

2. **Zero or Negative Price**
   ```typescript
   // Test with invalid price value
   it("Should reject operation with price <= 0", async () => {
     // Mock Pyth account with price = 0 or negative
     // Expected error: "Invalid amount" (from price > 0 check)
   });
   ```

3. **Invalid Pyth Account**
   ```typescript
   // Test with wrong Pyth price feed
   it("Should reject operation with invalid price feed", async () => {
     await protocolProgram.methods.openTrove(...)
       .accounts({
         ...accounts,
         pythPriceAccount: PYTH_PRICE_FEEDS.ETH,  // Wrong feed for SOL
       })
       .signers([user1])
       .rpc();
     // Expected: CPI error or price mismatch
   });
   ```

4. **Oracle Program Mismatch**
   ```typescript
   // Test with wrong oracle program
   it("Should reject CPI to wrong oracle program", async () => {
     await protocolProgram.methods.openTrove(...)
       .accounts({
         ...accounts,
         oracleProgram: wrongProgramId,  // Not the authorized oracle
       })
       .signers([user1])
       .rpc();
     // Expected error: Program mismatch or CPI failure
   });
   ```

### **CPI Integration Checklist**

#### **For Each Test File**
- [ ] Import `SYSVAR_CLOCK_PUBKEY` from `@solana/web3.js`
- [ ] Define `PYTH_PRICE_FEEDS` constants for all collateral types
- [ ] Update all protocol instruction calls to include:
  - [ ] `pythPriceAccount` (matching collateral denom)
  - [ ] `clock: SYSVAR_CLOCK_PUBKEY`
- [ ] Add error handling tests for CPI failures
- [ ] Verify ICR calculations use real oracle prices

#### **Oracle State Setup**
- [ ] Ensure oracle is initialized before protocol tests
- [ ] Set collateral data with correct Pyth price IDs:
  ```typescript
  await oracleProgram.methods.setData({
    denom: "SOL",
    decimal: 9,
    priceId: PYTH_PRICE_FEEDS.SOL.toBuffer().toString('hex'),
    pythPriceAccount: PYTH_PRICE_FEEDS.SOL,
  }).accounts({...}).rpc();
  ```
- [ ] Verify `getAllDenoms()` returns configured collaterals

### **Debugging CPI Issues**

#### **Common Errors**

1. **Missing Account Error**
   ```
   Error: Missing account: pyth_price_account
   ```
   **Solution**: Add `pythPriceAccount: PYTH_PRICE_FEEDS.<DENOM>` to accounts

2. **Clock Sysvar Error**
   ```
   Error: Missing account: clock
   ```
   **Solution**: Add `clock: SYSVAR_CLOCK_PUBKEY` to accounts

3. **Stale Price Error**
   ```
   Error: Invalid amount (code: 6004)
   ```
   **Solution**: Price timestamp is >5 minutes old. Ensure Pyth price feed is recent

4. **Invalid Price Error**
   ```
   Error: Invalid amount (code: 6004)
   ```
   **Solution**: Price is ≤ 0. Verify Pyth price feed has valid price data

5. **CPI Return Data Error**
   ```
   Error: Failed to deserialize return data
   ```
   **Solution**: Verify oracle program is deployed and `get_price` returns correct PriceResponse

#### **Debugging Commands**
```bash
# Check oracle state
solana account <ORACLE_STATE_PUBKEY> --output json

# Verify Pyth price feed (Devnet)
solana account J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix --output json

# Run with verbose logs
RUST_LOG=trace anchor test

# Simulate instruction (no execution)
anchor test --skip-deploy -- --grep "open_trove" --simulate
```

### **Integration Test Coverage**

#### **Happy Path Tests** ✅
- [ ] Open trove with valid oracle price
- [ ] Add collateral updates ICR correctly
- [ ] Borrow loan validates with current price
- [ ] Liquidation uses accurate prices

#### **Error Path Tests** ⚠️
- [ ] Stale price rejection (>5 minutes old)
- [ ] Zero or negative price rejection
- [ ] Invalid Pyth account handling
- [ ] Oracle program validation

#### **Edge Cases** 🔍
- [ ] Price volatility during operations
- [ ] Multiple collateral types with different oracles
- [ ] Oracle state changes mid-transaction
- [ ] Pyth network downtime scenarios

### **Performance Considerations**

- **CPI Overhead**: Oracle CPI adds ~5000 CU per call
- **Account Verification**: Pyth account deserialization adds ~2000 CU
- **Total Impact**: ~7000 CU per price lookup (acceptable)

### **Security Testing**

1. **Price Manipulation**: Verify CPI uses on-chain Pyth data (not user-provided)
2. **Authorization**: Ensure only authorized oracle program can be called
3. **Staleness**: Confirm 5-minute (300 second) staleness check is enforced
4. **Price Validity**: Ensure price > 0 validation is enforced

---

## 📞 **Support & Resources**

### **Useful Commands**
```bash
# Check program status
solana program show <PROGRAM_ID>

# Check account balance
solana balance

# View transaction logs
solana confirm <TRANSACTION_SIGNATURE>

# Get account info
solana account <ACCOUNT_ADDRESS>
```

### **Documentation**
- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Documentation](https://docs.solana.com/)
- [SPL Token Documentation](https://spl.solana.com/token)

---

**Status**: 🚀 **Ready for Comprehensive Testing**  
**Next Action**: Execute basic functionality tests  
**Timeline**: Immediate testing phase 