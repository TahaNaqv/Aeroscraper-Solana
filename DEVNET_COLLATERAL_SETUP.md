# Devnet Collateral Setup Guide

## Critical Information for Testing on Devnet

### The Collateral Mint Problem

When testing the Aerospacer Protocol on devnet, you **MUST** use the existing collateral mints that are already associated with the protocol vaults. Creating new collateral mints will cause constraint violations because:

1. The `protocol_collateral_vault` PDA is derived from the collateral denomination (e.g., "SOL")
2. Each vault PDA can only be initialized once with a specific mint
3. Anchor's `token::mint = collateral_mint` constraint enforces that the provided mint matches the vault's mint
4. If you provide a different mint than what's already in the vault, you get error `ConstraintTokenMint` (Error Number: 2014)

### Error Example

```
❌ AnchorError caused by account: protocol_collateral_account. 
Error Code: ConstraintTokenMint. Error Number: 2014. 
Error Message: A token mint constraint was violated.
Program log: Left: Hygyfy8RBxLvoz5b3ffsg9PAvEkT3BJXXdTpVu6ftZYz (existing vault mint)
Program log: Right: BfUm4VuvhC3j5n1XJwibxRmXxZhL1G6wizRQ9KWZpCrE (your new mint)
```

## How to Fetch the Correct Collateral Mint

### Method 1: Fetch from Protocol Vault (Recommended)

```typescript
// Derive the vault PDA for the collateral denomination
const [protocolCollateralVaultPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("protocol_collateral_vault"), Buffer.from("SOL")],
  protocolProgram.programId
);

// Check if vault exists on devnet
const vaultAccountInfo = await provider.connection.getAccountInfo(protocolCollateralVaultPda);
if (vaultAccountInfo) {
  // Vault exists - fetch its mint address
  const vaultAccount = await provider.connection.getParsedAccountInfo(protocolCollateralVaultPda);
  if (vaultAccount.value && 'parsed' in vaultAccount.value.data) {
    collateralMint = new PublicKey(vaultAccount.value.data.parsed.info.mint);
    console.log("✅ Using existing devnet collateral mint:", collateralMint.toString());
  }
} else {
  // Vault doesn't exist - create new mint (localnet scenario)
  collateralMint = await createMint(provider.connection, adminKeypair, admin.publicKey, null, 9);
  console.log("✅ Created new collateral mint for localnet:", collateralMint.toString());
}
```

### Method 2: Query from Oracle Contract

The oracle contract stores price information for each collateral denomination, which includes the mint address.

```typescript
// Query oracle for collateral info
const collateralInfo = await oracleProgram.methods
  .getAllDenoms()
  .accounts({
    state: oracleState,
  })
  .view();

// Find SOL collateral and get its mint
const solCollateral = collateralInfo.find(c => c.denom === "SOL");
if (solCollateral) {
  collateralMint = solCollateral.mint;
}
```

## Devnet Collateral Mints (as of deployment)

The following collateral mints are currently configured on devnet for the Aerospacer Protocol:

### SOL Collateral
- **Denomination**: "SOL"
- **Mint Address**: `Hygyfy8RBxLvoz5b3ffsg9PAvEkT3BJXXdTpVu6ftZYz`
- **Decimals**: 9
- **Vault PDA**: Derived from `["protocol_collateral_vault", "SOL"]`

> **Note**: This mint address is read from the actual devnet vault. If you're testing on a different devnet deployment, use the vault-fetch method above to get the correct mint.

## Test File Updates

### Fixed: `tests/protocol-core.ts`

This test file has been updated to:
1. Derive the `protocol_collateral_vault` PDA for "SOL"
2. Fetch the existing vault account from devnet
3. Parse the vault's mint address
4. Use that mint for all collateral operations
5. Handle the case where we can't mint tokens (devnet vs localnet)

### Working Tests (No Changes Needed)

These test files work correctly because they don't perform trove operations that require matching collateral mints:
- `tests/protocol-simple-test.ts` - Only checks initialization
- `tests/protocol-initialization.ts` - Only verifies protocol state

## Token Minting Considerations

### On Localnet
When testing on localnet with a fresh deployment:
- You create your own collateral mint
- You control the mint authority
- You can mint tokens freely for testing

### On Devnet
When testing on devnet with existing deployments:
- The collateral mint already exists
- You likely DON'T control the mint authority
- You CANNOT mint new tokens
- Users need to obtain tokens through:
  - Airdrops (if mint supports it)
  - Transfers from accounts that have tokens
  - DEX swaps or faucets

The test file now checks mint authority and only attempts to mint if you control it:

```typescript
const mintInfo = await provider.connection.getParsedAccountInfo(collateralMint);
let canMint = false;
if (mintInfo.value && 'parsed' in mintInfo.value.data) {
  const mintAuthority = mintInfo.value.data.parsed.info.mintAuthority;
  canMint = mintAuthority && new PublicKey(mintAuthority).equals(admin.publicKey);
}

if (canMint) {
  // Mint tokens for testing
} else {
  // Check user balances and warn if insufficient
}
```

## Best Practices for Devnet Testing

1. **Always fetch existing mints** - Never assume you can create new mints for existing vaults
2. **Check token balances** - Ensure test users have sufficient collateral before running tests
3. **Handle both scenarios** - Write tests that work on both localnet (fresh) and devnet (existing)
4. **Document mint addresses** - Keep track of which mints are used for which collaterals
5. **Use try-catch blocks** - Handle cases where accounts may or may not exist

## Troubleshooting

### Error: `ConstraintTokenMint` (2014)
**Cause**: Providing a different collateral mint than what's stored in the vault  
**Solution**: Fetch the mint from the existing vault PDA (see Method 1 above)

### Error: `AccountNotInitialized` (3012) for `user_debt_amount`
**Cause**: Trying to add collateral or borrow before opening a trove  
**Solution**: Ensure `openTrove` is called successfully first

### Error: `InsufficientCollateral` (6013)
**Cause**: User doesn't have enough collateral tokens in their account  
**Solution**: Ensure users have tokens before running tests (mint on localnet, or transfer on devnet)

### Warning: Cannot mint tokens on devnet
**Cause**: The collateral mint authority is not your test admin  
**Solution**: This is expected on devnet. Ensure test users are pre-funded with collateral tokens

## Summary

The key takeaway: **On devnet, collateral mints are immutable per denomination**. Your tests must adapt to the existing infrastructure rather than trying to create new mints. The vault PDA system ensures deterministic addresses but also locks in the mint address when first initialized.
