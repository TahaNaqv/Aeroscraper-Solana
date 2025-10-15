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

// Constants
export const PYTH_ORACLE_ADDRESS = new PublicKey("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
export const SOL_DENOM = "SOL";
export const SCALE_FACTOR = new BN("1000000000000000000"); // 10^18
export const MIN_LOAN_AMOUNT = SCALE_FACTOR; // 1 aUSD
export const MIN_COLLATERAL_RATIO = 115; // 115%
export const LIQUIDATION_THRESHOLD = 110; // 110%

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
  const [protocolStablecoinVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("protocol_stablecoin_vault")],
    programId
  );

  const [protocolCollateralVault] = PublicKey.findProgramAddressSync(
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
    [Buffer.from("trove_node"), user.toBuffer()],
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

  const [totalLiquidationCollateralGain] = PublicKey.findProgramAddressSync(
    [Buffer.from("total_liquidation_collateral_gain"), Buffer.from(collateralDenom)],
    programId
  );

  return {
    protocolStablecoinVault,
    protocolCollateralVault,
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
  };
}

// Setup test environment
export async function setupTestEnvironment(): Promise<TestContext> {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  const admin = provider.wallet as anchor.Wallet;

  // Create mints
  const stablecoinMint = await createMint(
    provider.connection,
    admin.payer,
    admin.publicKey,
    null,
    18
  );

  const collateralMint = await createMint(
    provider.connection,
    admin.payer,
    admin.publicKey,
    null,
    9
  );

  // Initialize oracle
  const oracleStateKeypair = Keypair.generate();
  await oracleProgram.methods
    .initialize({ oracleAddress: PYTH_ORACLE_ADDRESS })
    .accounts({
      state: oracleStateKeypair.publicKey,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    })
    .signers([oracleStateKeypair])
    .rpc();

  // Initialize fees
  const feeStateKeypair = Keypair.generate();
  await feesProgram.methods
    .initialize({
      admin: admin.publicKey,
      feeAddress1: admin.publicKey,
      feeAddress2: admin.publicKey,
    })
    .accounts({
      state: feeStateKeypair.publicKey,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([feeStateKeypair])
    .rpc();

  // Initialize protocol
  const protocolStateKeypair = Keypair.generate();
  const sortedTrovesState = derivePDAs(SOL_DENOM, admin.publicKey, protocolProgram.programId).sortedTrovesState;

  await protocolProgram.methods
    .initialize({
      stablecoinMintAddress: stablecoinMint,
      collateralMintAddress: collateralMint,
      minCollateralRatio: MIN_COLLATERAL_RATIO,
      oracleHelperAddr: oracleProgram.programId,
      feeHelperAddr: feesProgram.programId,
      oracleStateAddr: oracleStateKeypair.publicKey,
      feeStateAddr: feeStateKeypair.publicKey,
    })
    .accounts({
      state: protocolStateKeypair.publicKey,
      admin: admin.publicKey,
      sortedTrovesState,
      systemProgram: SystemProgram.programId,
    })
    .signers([protocolStateKeypair])
    .rpc();

  return {
    provider,
    protocolProgram,
    oracleProgram,
    feesProgram,
    admin,
    stablecoinMint,
    collateralMint,
    protocolState: protocolStateKeypair.publicKey,
    oracleState: oracleStateKeypair.publicKey,
    feeState: feeStateKeypair.publicKey,
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

  // Airdrop SOL
  await provider.connection.requestAirdrop(user.publicKey, 5 * LAMPORTS_PER_SOL);
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Create token account and fund it
  const collateralAccount = await createAssociatedTokenAccount(
    provider.connection,
    user,
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

  const userStablecoinAccount = await createAssociatedTokenAccount(
    ctx.provider.connection,
    user,
    ctx.stablecoinMint,
    user.publicKey
  );

  await ctx.protocolProgram.methods
    .openTrove({
      collateralAmount,
      loanAmount,
      collateralDenom,
    })
    .accounts({
      user: user.publicKey,
      state: ctx.protocolState,
      userDebtAmount: pdas.userDebtAmount,
      userCollateralAmount: pdas.userCollateralAmount,
      liquidityThreshold: pdas.liquidityThreshold,
      node: pdas.node,
      sortedTrovesState: pdas.sortedTrovesState,
      totalCollateralAmount: pdas.totalCollateralAmount,
      stableCoinMint: ctx.stablecoinMint,
      collateralMint: ctx.collateralMint,
      userCollateralAccount,
      userStablecoinAccount,
      protocolStablecoinVault: pdas.protocolStablecoinVault,
      protocolCollateralVault: pdas.protocolCollateralVault,
      oracleProgram: ctx.oracleProgram.programId,
      oracleState: ctx.oracleState,
      pythPriceAccount: PYTH_ORACLE_ADDRESS,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
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

  await ctx.protocolProgram.methods
    .stake({ amount: ausdAmount })
    .accounts({
      user: user.publicKey,
      state: ctx.protocolState,
      userStakeAmount: pdas.userStakeAmount,
      stableCoinMint: ctx.stablecoinMint,
      userStablecoinAccount,
      protocolStablecoinVault: pdas.protocolStablecoinVault,
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
