import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  createMint,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
  mintTo,
  TOKEN_PROGRAM_ID,
  getAccount,
} from "@solana/spl-token";
import * as fs from "fs";
import * as path from "path";

// Constants
export const PYTH_ORACLE_ADDRESS = new PublicKey("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
export const SOL_DENOM = "SOL";
export const SCALE_FACTOR = new BN("1000000000000000000"); // 10^18
export const MIN_LOAN_AMOUNT = SCALE_FACTOR; // 1 aUSD
export const MIN_COLLATERAL_RATIO = 115; // 115%
export const LIQUIDATION_THRESHOLD = 110; // 110%

// Load fixed test user keypairs (avoids Node PDA collision on devnet)
export function loadFixedKeypair(filename: string): Keypair {
  const keypairPath = path.join(__dirname, "..", "keys", filename);
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf8"));
  return Keypair.fromSecretKey(new Uint8Array(keypairData));
}

// Load user1 and user2 keypairs (standardized across all tests)
export function loadTestUsers(): { user1: Keypair; user2: Keypair } {
  return {
    user1: loadFixedKeypair("user1-keypair.json"),
    user2: loadFixedKeypair("user2-keypair.json"),
  };
}

// Test context for protocol testing
export interface TestContext {
  provider: anchor.AnchorProvider;
  protocolProgram: Program<AerospacerProtocol>;
  oracleProgram: Program<AerospacerOracle>;
  feesProgram: Program<AerospacerFees>;
  admin: anchor.Wallet;
  stablecoinMint: PublicKey;
  collateralMint: PublicKey;
  protocolState: PublicKey;
  oracleState: PublicKey;
  feeState: PublicKey;
  sortedTrovesState: PublicKey;
}

// Helper to derive PDA addresses
export function derivePDAs(collateralDenom: string, user: PublicKey, programId: PublicKey) {
  const [protocolStablecoinAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("protocol_stablecoin_vault")],
    programId
  );

  const [protocolCollateralAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("protocol_collateral_vault"), Buffer.from(collateralDenom)],
    programId
  );

  const [totalCollateralAmount] = PublicKey.findProgramAddressSync(
    [Buffer.from("total_collateral_amount"), Buffer.from(collateralDenom)],
    programId
  );

  const [userDebtAmount] = PublicKey.findProgramAddressSync(
    [Buffer.from("user_debt_amount"), user.toBuffer()],
    programId
  );

  const [userCollateralAmount] = PublicKey.findProgramAddressSync(
    [Buffer.from("user_collateral_amount"), user.toBuffer(), Buffer.from(collateralDenom)],
    programId
  );

  const [liquidityThreshold] = PublicKey.findProgramAddressSync(
    [Buffer.from("liquidity_threshold"), user.toBuffer()],
    programId
  );

  const [node] = PublicKey.findProgramAddressSync(
    [Buffer.from("node"), user.toBuffer()],
    programId
  );

  const [sortedTrovesState] = PublicKey.findProgramAddressSync(
    [Buffer.from("sorted_troves_state")],
    programId
  );

  const [userStakeAmount] = PublicKey.findProgramAddressSync(
    [Buffer.from("user_stake_amount"), user.toBuffer()],
    programId
  );

  const [stabilityPoolSnapshot] = PublicKey.findProgramAddressSync(
    [Buffer.from("stability_pool_snapshot"), Buffer.from(collateralDenom)],
    programId
  );

  const [userCollateralSnapshot] = PublicKey.findProgramAddressSync(
    [Buffer.from("user_collateral_snapshot"), user.toBuffer(), Buffer.from(collateralDenom)],
    programId
  );

  // Note: total_liq_gain requires block_height parameter in Rust, not used in basic setup
  // Removed to prevent "Max seed length exceeded" error
  const totalLiquidationCollateralGain = PublicKey.default;

  // Fee-related PDAs
  const [stabilityPoolTokenAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("stability_pool_token_account")],
    programId
  );

  const [feeAddress1TokenAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("fee_address_1_token_account")],
    programId
  );

  const [feeAddress2TokenAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("fee_address_2_token_account")],
    programId
  );

  return {
    protocolStablecoinAccount,
    protocolCollateralAccount,
    totalCollateralAmount,
    userDebtAmount,
    userCollateralAmount,
    liquidityThreshold,
    node,
    sortedTrovesState,
    userStakeAmount,
    stabilityPoolSnapshot,
    userCollateralSnapshot,
    totalLiquidationCollateralGain,
    stabilityPoolTokenAccount,
    feeAddress1TokenAccount,
    feeAddress2TokenAccount,
  };
}

// Setup test environment
export async function setupTestEnvironment(): Promise<TestContext> {
  console.log("🚀 Setting up test environment for devnet...");
  
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  const admin = provider.wallet as anchor.Wallet;

  // Create stablecoin mint
  const stablecoinMint = await createMint(
    provider.connection,
    admin.payer,
    admin.publicKey,
    null,
    18
  );

  // Check if protocol_collateral_vault exists on devnet and extract mint if so
  let collateralMint: PublicKey;
  const [protocolCollateralVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("protocol_collateral_vault"), Buffer.from(SOL_DENOM)],
    protocolProgram.programId
  );

  try {
    const vaultAccount = await provider.connection.getParsedAccountInfo(protocolCollateralVaultPda);
    if (vaultAccount.value && 'parsed' in vaultAccount.value.data) {
      collateralMint = new PublicKey(vaultAccount.value.data.parsed.info.mint);
      console.log("✅ Using existing devnet collateral mint from vault:", collateralMint.toString());
    } else {
      throw new Error("Failed to parse vault account data");
    }
  } catch (error) {
    // Vault doesn't exist - create a new mint (localnet scenario)
    collateralMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      9
    );
    console.log("✅ Created new collateral mint for localnet:", collateralMint.toString());
  }

  // Initialize oracle using PDA
  const [oracleStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("state")],
    oracleProgram.programId
  );

  try {
    const existingState = await oracleProgram.account.oracleStateAccount.fetch(oracleStatePDA);
    console.log("✅ Oracle already initialized on devnet");
  } catch (error) {
    console.log("Initializing oracle...");
    await oracleProgram.methods
      .initialize({ oracleAddress: PYTH_ORACLE_ADDRESS })
      .accounts({
        state: oracleStatePDA,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      })
      .signers([admin.payer])
      .rpc();
    console.log("✅ Oracle initialized");
  }

  // Initialize fees using PDA
  const [feesStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("fee_state")],
    feesProgram.programId
  );

  try {
    const existingState = await feesProgram.account.feeStateAccount.fetch(feesStatePDA);
    console.log("✅ Fees already initialized on devnet");
  } catch (error) {
    console.log("Initializing fees...");
    await feesProgram.methods
      .initialize()
      .accounts({
        state: feesStatePDA,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin.payer])
      .rpc();
    console.log("✅ Fees initialized");
  }

  // Initialize protocol using PDA
  const [protocolStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("state")],
    protocolProgram.programId
  );

  try {
    const existingState = await protocolProgram.account.stateAccount.fetch(protocolStatePDA);
    console.log("✅ Protocol already initialized on devnet");
  } catch (error) {
    console.log("Initializing protocol...");
    await protocolProgram.methods
      .initialize({
        stableCoinCodeId: new anchor.BN(1),
        oracleHelperAddr: oracleProgram.programId,
        oracleStateAddr: oracleStatePDA,
        feeDistributorAddr: feesProgram.programId,
        feeStateAddr: feesStatePDA,
      })
      .accounts({
        state: protocolStatePDA,
        admin: admin.publicKey,
        stableCoinMint: stablecoinMint,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin.payer])
      .rpc();
    console.log("✅ Protocol initialized");
  }

  const sortedTrovesState = derivePDAs(SOL_DENOM, admin.publicKey, protocolProgram.programId).sortedTrovesState;

  return {
    provider,
    protocolProgram,
    oracleProgram,
    feesProgram,
    admin,
    stablecoinMint,
    collateralMint,
    protocolState: protocolStatePDA,
    oracleState: oracleStatePDA,
    feeState: feesStatePDA,
    sortedTrovesState,
  };
}

// Helper to create and fund a test user
export async function createTestUser(
  provider: anchor.AnchorProvider,
  collateralMint: PublicKey,
  amount: BN
): Promise<{ user: Keypair; collateralAccount: PublicKey }> {
  const user = Keypair.generate();

  // Transfer enough SOL for transaction fees and account rent (0.02 SOL = 20M lamports)
  // Node account creation requires ~1.28M lamports rent-exempt minimum + transaction fees
  const transferAmount = 20_000_000; // 0.02 SOL in lamports (was 0.001 SOL, increased for buffer)
  const transferTx = new anchor.web3.Transaction().add(
    anchor.web3.SystemProgram.transfer({
      fromPubkey: provider.wallet.publicKey,
      toPubkey: user.publicKey,
      lamports: transferAmount,
    })
  );
  
  await provider.sendAndConfirm(transferTx, [provider.wallet.payer]);

  // Create token account and fund it
  const collateralAccount = await createAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    collateralMint,
    user.publicKey
  );

  // Mint collateral to user (admin mints)
  await mintTo(
    provider.connection,
    provider.wallet.payer,
    collateralMint,
    collateralAccount,
    provider.wallet.publicKey,
    amount.toNumber()
  );

  return { user, collateralAccount };
}

// Helper to open a trove for a user
export async function openTroveForUser(
  ctx: TestContext,
  user: Keypair,
  collateralAmount: BN,
  loanAmount: BN,
  collateralDenom: string
): Promise<void> {
  const pdas = derivePDAs(collateralDenom, user.publicKey, ctx.protocolProgram.programId);

  const userCollateralAccount = await getAssociatedTokenAddress(
    ctx.collateralMint,
    user.publicKey
  );

  const userStablecoinAccount = await getAssociatedTokenAddress(
    ctx.stablecoinMint,
    user.publicKey
  );

  // Create token accounts if they don't exist
  try {
    await createAssociatedTokenAccount(
      ctx.provider.connection,
      ctx.admin.payer,
      ctx.stablecoinMint,
      user.publicKey
    );
  } catch (error) {
    // Account might already exist
  }

  await ctx.protocolProgram.methods
    .openTrove({
      loanAmount,
      collateralDenom,
      collateralAmount,
    })
    .accounts({
      user: user.publicKey,
      userDebtAmount: pdas.userDebtAmount,
      liquidityThreshold: pdas.liquidityThreshold,
      userCollateralAmount: pdas.userCollateralAmount,
      userCollateralAccount,
      collateralMint: ctx.collateralMint,
      protocolCollateralAccount: pdas.protocolCollateralAccount,
      totalCollateralAmount: pdas.totalCollateralAmount,
      sortedTrovesState: pdas.sortedTrovesState,
      node: pdas.node,
      state: ctx.protocolState,
      userStablecoinAccount,
      protocolStablecoinAccount: pdas.protocolStablecoinAccount,
      stableCoinMint: ctx.stablecoinMint,
      oracleProgram: ctx.oracleProgram.programId,
      oracleState: ctx.oracleState,
      pythPriceAccount: PYTH_ORACLE_ADDRESS,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      feesProgram: ctx.feesProgram.programId,
      feesState: ctx.feeState,
      stabilityPoolTokenAccount: pdas.stabilityPoolTokenAccount,
      feeAddress1TokenAccount: pdas.feeAddress1TokenAccount,
      feeAddress2TokenAccount: pdas.feeAddress2TokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([user])
    .rpc();
}

// Helper to check account balance
export async function getTokenBalance(
  connection: anchor.web3.Connection,
  tokenAccount: PublicKey
): Promise<BN> {
  const accountInfo = await getAccount(connection, tokenAccount);
  return new BN(accountInfo.amount.toString());
}

// Helper to fetch protocol state account
export async function fetchUserDebtAmount(
  program: Program<AerospacerProtocol>,
  userDebtAmountPDA: PublicKey
): Promise<any> {
  return await program.account.userDebtAmount.fetch(userDebtAmountPDA);
}

export async function fetchUserCollateralAmount(
  program: Program<AerospacerProtocol>,
  userCollateralAmountPDA: PublicKey
): Promise<any> {
  return await program.account.userCollateralAmount.fetch(userCollateralAmountPDA);
}

export async function fetchLiquidityThreshold(
  program: Program<AerospacerProtocol>,
  liquidityThresholdPDA: PublicKey
): Promise<any> {
  return await program.account.liquidityThreshold.fetch(liquidityThresholdPDA);
}

// Helper to calculate ICR
export function calculateICR(collateralValue: BN, debtValue: BN): number {
  if (debtValue.isZero()) return Infinity;
  return collateralValue.mul(new BN(100)).div(debtValue).toNumber();
}

// Helper to stake aUSD in stability pool (for liquidation absorber)
export async function stakeInStabilityPool(
  ctx: TestContext,
  user: Keypair,
  ausdAmount: BN
): Promise<void> {
  const pdas = derivePDAs(SOL_DENOM, user.publicKey, ctx.protocolProgram.programId);
  
  const userStablecoinAccount = await getAssociatedTokenAddress(
    ctx.stablecoinMint,
    user.publicKey
  );

  // Create token account if it doesn't exist
  try {
    await createAssociatedTokenAccount(
      ctx.provider.connection,
      ctx.admin.payer,
      ctx.stablecoinMint,
      user.publicKey
    );
  } catch (error) {
    // Account might already exist
  }

  await ctx.protocolProgram.methods
    .stake({ amount: ausdAmount })
    .accounts({
      user: user.publicKey,
      userStakeAmount: pdas.userStakeAmount,
      state: ctx.protocolState,
      userStablecoinAccount,
      protocolStablecoinAccount: pdas.protocolStablecoinAccount,
      stableCoinMint: ctx.stablecoinMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([user])
    .rpc();
}

// Helper to create a near-liquidatable trove (ICR close to liquidation threshold)
export async function createLiquidatableTrove(
  ctx: TestContext,
  user: Keypair,
  collateralDenom: string
): Promise<void> {
  // Create trove with 120% ICR (above 115% minimum, but vulnerable)
  // Would become liquidatable if:
  // - Collateral price drops ~9% (120% → 110%)
  // - Fees accrue over time
  const collateralAmount = new BN(6_000_000_000); // 6 SOL
  const loanAmount = SCALE_FACTOR.mul(new BN(100)); // 100 aUSD
  
  // At $200/SOL: 6 SOL * $200 = $1200 collateral, $100 debt = 120% ICR
  // This passes the 115% minimum but is close to liquidation threshold
  await openTroveForUser(ctx, user, collateralAmount, loanAmount, collateralDenom);
}

// Helper to create a healthy trove for redemption target
export async function createRedeemableTrove(
  ctx: TestContext,
  user: Keypair,
  collateralAmount: BN,
  loanAmount: BN,
  collateralDenom: string
): Promise<void> {
  await openTroveForUser(ctx, user, collateralAmount, loanAmount, collateralDenom);
}
