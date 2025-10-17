# Aerospacer Protocol - Deployment Checklist

## 🎯 Pre-Deployment Validation

Use this checklist to ensure the protocol is ready for devnet/mainnet deployment.

---

## ✅ Phase 1: Local Environment Setup (Estimated: 2-3 hours)

### 1.1 Development Environment
- [ ] Rust installed (v1.85.0+)
- [ ] Solana CLI installed (v2.1.15+)
- [ ] Anchor CLI installed (v0.31.1)
- [ ] Node.js installed (v20.0.0+)
- [ ] Dependencies installed (`npm install`)

### 1.2 Build Verification
- [ ] `anchor build` completes successfully
- [ ] All 3 programs compile without errors:
  - [ ] aerospacer-protocol
  - [ ] aerospacer-oracle
  - [ ] aerospacer-fees
- [ ] TypeScript types generated in `target/types/`
- [ ] IDL files generated in `target/idl/`

### 1.3 Local Validator Setup
- [ ] `solana-test-validator` starts successfully
- [ ] Test keypair generated
- [ ] Test SOL airdropped (10+ SOL)
- [ ] Programs deployed to local validator

**Expected Time**: 1-2 hours

---

## ✅ Phase 2: Test Execution (Estimated: 4-6 hours)

### 2.1 Protocol Core Tests
- [ ] **Initialization** (`protocol-initialization.ts`)
  - [ ] State initialized with correct parameters
  - [ ] P factor = 10^18
  - [ ] Epoch = 0
  - [ ] MCR = 115%, Protocol fee = 5%

- [ ] **Trove Management** (`protocol-trove-management.ts`)
  - [ ] Open trove with valid collateral
  - [ ] Add collateral (ICR increases)
  - [ ] Remove collateral (ICR validation)
  - [ ] Borrow loan (mint stablecoin)
  - [ ] Repay loan (burn stablecoin)
  - [ ] Close trove (account cleanup)

- [ ] **Stability Pool** (`protocol-stability-pool.ts`)
  - [ ] Stake aUSD to pool
  - [ ] Unstake from pool
  - [ ] Compounded stake calculation (P factor)
  - [ ] User snapshots (P & S)
  - [ ] Withdraw liquidation gains

### 2.2 Critical Operations Tests
- [ ] **Liquidation** (`protocol-liquidation.ts`)
  - [ ] Query liquidatable troves
  - [ ] Single trove liquidation
  - [ ] Batch liquidation (up to 50 troves)
  - [ ] P factor depletion on liquidation
  - [ ] S factor accumulation (gains)
  - [ ] Debt burning from stability pool
  - [ ] Collateral distribution to stakers

- [ ] **Redemption** (`protocol-redemption.ts`)
  - [ ] Sorted list traversal (tail→head)
  - [ ] Partial redemption (multiple troves)
  - [ ] Full redemption (single trove)
  - [ ] Redemption fee calculation
  - [ ] State cleanup after redemption
  - [ ] Insufficient liquidity rejection

- [ ] **Sorted Troves** (`protocol-sorted-troves.ts`)
  - [ ] Insert at correct ICR position
  - [ ] Remove and update pointers
  - [ ] Reinsert after ICR change
  - [ ] Head/tail pointer management
  - [ ] List integrity maintained

### 2.3 Integration Tests
- [ ] **Oracle Integration** (`protocol-oracle-integration.ts`)
  - [ ] CPI to Pyth Network successful
  - [ ] Real-time price feeds working
  - [ ] ICR calculated with live prices
  - [ ] Price staleness check (5 min)
  - [ ] Invalid oracle rejection

- [ ] **Fee Distribution** (`protocol-fees-integration.ts`)
  - [ ] CPI to fees program successful
  - [ ] Stability pool mode distribution
  - [ ] Treasury mode (50/50 split)
  - [ ] Fee calculation accuracy

### 2.4 Security & Edge Cases
- [ ] **Security Tests** (`protocol-security.ts`)
  - [ ] Admin-only operations enforced
  - [ ] PDA validation working
  - [ ] Forged account rejection
  - [ ] Unauthorized access blocked

- [ ] **Edge Cases** (`protocol-edge-cases.ts`)
  - [ ] Boundary conditions handled
  - [ ] Zero values rejected
  - [ ] Maximum values handled
  - [ ] Overflow prevention verified

- [ ] **Error Coverage** (`protocol-error-coverage.ts`)
  - [ ] All 20 error types tested
  - [ ] Error messages clear and actionable

### 2.5 Multi-User & Stress Tests
- [ ] **Multi-User** (`protocol-multi-user.ts`)
  - [ ] Concurrent trove operations
  - [ ] Multiple stakers in pool
  - [ ] Race condition handling

- [ ] **Stress Test** (`protocol-stress-test.ts`)
  - [ ] Maximum batch size (50 troves)
  - [ ] Large sorted list (100+ nodes)
  - [ ] Gas optimization verified
  - [ ] Performance acceptable

**Expected Time**: 3-4 hours

---

## ✅ Phase 3: Devnet Deployment (Estimated: 3-4 hours)

### 3.1 Devnet Configuration
- [ ] Solana CLI set to devnet (`solana config set --url devnet`)
- [ ] Devnet SOL airdropped (5+ SOL)
- [ ] Deployment keypair secured
- [ ] Program IDs verified in `Anchor.toml`

### 3.2 Program Deployment
- [ ] Deploy oracle program (`anchor deploy -p aerospacer-oracle`)
- [ ] Deploy fees program (`anchor deploy -p aerospacer-fees`)
- [ ] Deploy protocol program (`anchor deploy -p aerospacer-protocol`)
- [ ] All programs deployed successfully
- [ ] Program IDs match `Anchor.toml`

### 3.3 Oracle Initialization on Devnet
- [ ] Run `npm run init_oracle_devnet`
- [ ] Oracle state initialized with Pyth addresses
- [ ] Supported assets added (SOL, USDC, etc.)
  - [ ] Run `npm run add_assets_devnet`
- [ ] Price feeds validated
  - [ ] Run `npm run test_prices_devnet`
- [ ] All collateral types returning valid prices

### 3.4 Protocol Initialization on Devnet
- [ ] Protocol state initialized
- [ ] Oracle addresses set correctly
- [ ] Fee program addresses set correctly
- [ ] Stablecoin mint created
- [ ] Initial parameters verified (MCR, fee %)

### 3.5 Devnet Testing
- [ ] Oracle tests pass (`npm run test-oracle-devnet`)
- [ ] Fee tests pass (`npm run test-fee-devnet`)
- [ ] Full protocol test suite passes
- [ ] End-to-end user journey validated:
  - [ ] Open trove → Borrow → Repay → Close
  - [ ] Stake → Liquidation → Withdraw gains
  - [ ] Redemption flow complete

**Expected Time**: 2-3 hours

---

## ✅ Phase 4: Security & Economic Review (Estimated: 2-3 weeks)

### 4.1 Code Audit Preparation
- [ ] Code freeze for audit
- [ ] All tests passing (100% pass rate)
- [ ] Documentation complete and accurate
- [ ] Audit firm selected and engaged
- [ ] Code submitted for audit

### 4.2 Security Audit Findings
- [ ] Critical vulnerabilities addressed (if any)
- [ ] High-priority issues fixed
- [ ] Medium-priority issues reviewed
- [ ] Low-priority issues documented
- [ ] Audit report published

### 4.3 Economic Validation
- [ ] Liquidation incentives validated
- [ ] Fee parameters optimized
- [ ] Redemption economics reviewed
- [ ] Game theory analysis complete
- [ ] Economic model stress-tested

### 4.4 Penetration Testing
- [ ] White-hat testing conducted
- [ ] Attack vectors identified and mitigated
- [ ] Bug bounty program prepared
- [ ] Security best practices documented

**Expected Time**: 2-3 weeks (external dependency)

---

## ✅ Phase 5: Mainnet Preparation (Estimated: 1 week)

### 5.1 Admin Setup
- [ ] Multi-sig wallet created (e.g., Squads Protocol)
- [ ] Admin keys secured (HSM or hardware wallet)
- [ ] Key ceremony documented
- [ ] Backup procedures established
- [ ] Emergency response plan documented

### 5.2 Monitoring & Alerts
- [ ] On-chain monitoring setup
- [ ] Price feed monitoring (Pyth staleness alerts)
- [ ] Liquidation queue monitoring
- [ ] Critical operation alerts configured
- [ ] Dashboard for key metrics

### 5.3 Documentation & Communication
- [ ] User documentation complete
- [ ] Developer documentation updated
- [ ] API documentation published
- [ ] Integration guides ready
- [ ] Community communication plan

### 5.4 Infrastructure
- [ ] RPC nodes configured (redundancy)
- [ ] Backup validators identified
- [ ] Rate limiting implemented
- [ ] DDoS protection configured
- [ ] Disaster recovery plan

**Expected Time**: 5-7 days

---

## ✅ Phase 6: Mainnet Deployment (Estimated: 1 day)

### 6.1 Final Validation
- [ ] All devnet tests passing (100%)
- [ ] Security audit complete and published
- [ ] Economic review complete
- [ ] Admin controls tested
- [ ] Emergency procedures rehearsed

### 6.2 Deployment Execution
- [ ] Solana CLI set to mainnet (`solana config set --url mainnet`)
- [ ] Mainnet SOL funded (deployment + buffer)
- [ ] Final code review (no changes since audit)
- [ ] Deploy oracle program
- [ ] Deploy fees program  
- [ ] Deploy protocol program
- [ ] Verify program IDs on-chain

### 6.3 Mainnet Initialization
- [ ] Initialize oracle with mainnet Pyth addresses
- [ ] Add supported collateral assets
- [ ] Validate price feeds (mainnet Pyth data)
- [ ] Initialize protocol state
- [ ] Set oracle and fee addresses
- [ ] Create stablecoin mint
- [ ] Transfer admin to multi-sig

### 6.4 Post-Deployment Validation
- [ ] All programs deployed successfully
- [ ] Oracle returning valid prices
- [ ] Test transaction on mainnet (small amount)
- [ ] Monitoring systems active
- [ ] Alert systems functional
- [ ] Team on standby for 24-48 hours

**Expected Time**: 4-8 hours

---

## ✅ Phase 7: Launch & Monitoring (Ongoing)

### 7.1 Soft Launch (Week 1)
- [ ] Limited user access (whitelist/caps)
- [ ] TVL capped for initial phase
- [ ] Close monitoring of all operations
- [ ] Daily health checks
- [ ] Performance metrics tracked

### 7.2 Gradual Scale-Up (Weeks 2-4)
- [ ] Increase TVL caps gradually
- [ ] Expand user access
- [ ] Monitor liquidation system
- [ ] Stability pool performance
- [ ] Redemption system usage

### 7.3 Full Production (Month 2+)
- [ ] Remove all caps
- [ ] Full public access
- [ ] Ongoing monitoring and optimization
- [ ] Regular security reviews
- [ ] Community governance transition

---

## 🚨 Emergency Procedures

### Circuit Breakers
- [ ] Protocol pause mechanism tested
- [ ] Emergency admin keys accessible
- [ ] Communication channels ready
- [ ] Response team identified
- [ ] Escalation procedures documented

### Incident Response
- [ ] Incident detection automated
- [ ] Response playbooks prepared
- [ ] Team on-call schedule
- [ ] Post-mortem process defined
- [ ] Transparency guidelines set

---

## 📊 Success Criteria

### Technical
- ✅ All tests passing (100%)
- ✅ Zero critical security issues
- ✅ Gas optimization complete
- ✅ Performance within targets
- ✅ Monitoring fully operational

### Economic
- ✅ Liquidation incentives validated
- ✅ Fee structure optimized
- ✅ Redemption economics sound
- ✅ Peg stability maintained
- ✅ Capital efficiency maximized

### Operational
- ✅ Multi-sig admin functional
- ✅ Emergency procedures tested
- ✅ Documentation complete
- ✅ Team trained and ready
- ✅ Community informed

---

## 📝 Sign-Off Required

Before mainnet deployment, obtain sign-off from:

- [ ] **Lead Developer** - Code complete and tested
- [ ] **Security Auditor** - No critical vulnerabilities
- [ ] **Economic Advisor** - Incentives validated
- [ ] **Operations Lead** - Infrastructure ready
- [ ] **Project Lead** - Final approval to deploy

---

## 🔗 Related Documentation

- **LOCAL_TESTING_GUIDE.md** - Local setup and testing
- **TEST_COVERAGE_ANALYSIS.md** - Detailed test coverage
- **replit.md** - Architecture and recent changes
- **PRODUCTION_READINESS_REPORT.md** - Comprehensive assessment

---

## 📞 Emergency Contacts

- **Protocol Lead**: [Contact Info]
- **Security Lead**: [Contact Info]
- **DevOps Lead**: [Contact Info]
- **Audit Firm**: [Contact Info]
- **Pyth Network Support**: [Contact Info]

---

**Last Updated**: October 17, 2025  
**Next Review**: Before Mainnet Deployment
