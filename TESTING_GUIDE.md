# Aerospacer Protocol - Devnet Testing Guide

## 🚀 **Testing Status**

**Current Status**: ✅ **Programs Deployed & Ready for Testing**  
**Network**: Solana Devnet  
**Programs**: 4/4 Deployed Successfully

---

## 📊 **Deployed Programs**

| Program | Program ID | Status |
|---------|------------|---------|
| **aerospacer_oracle** | `8gLDpdg9tFAtAnpZabd2w6V7qQikfqCKzPiNpx9Wcr3c` | ✅ Live |
| **aerospacer_protocol** | `mR3CUXYeYLjoxFJ1ieBfC9rLciZwe8feFYvXKdafihD` | ✅ Live |
| **aerospacer_fees** | `8PC52W8S5WQ1X6gBBNQr5AvYYxVEa68DahEgFJAueZF4` | ✅ Live |
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
solana program show 8gLDpdg9tFAtAnpZabd2w6V7qQikfqCKzPiNpx9Wcr3c

# Check Protocol Program
solana program show mR3CUXYeYLjoxFJ1ieBfC9rLciZwe8feFYvXKdafihD

# Check Fees Program
solana program show 8PC52W8S5WQ1X6gBBNQr5AvYYxVEa68DahEgFJAueZF4
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