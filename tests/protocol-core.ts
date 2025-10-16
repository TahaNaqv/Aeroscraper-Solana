import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY, LAMPORTS_PER_SOL } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccount,
  createMint,
  mintTo,
  transfer
} from "@solana/spl-token";
import { assert } from "chai";

// Constants
const PYTH_ORACLE_ADDRESS = new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");

describe("Aeroscraper Protocol Core Operations", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  // Test accounts
  const admin = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();

  // Token mints
  let stablecoinMint: PublicKey;
  let collateralMint: PublicKey;

  // Token accounts
  let adminStablecoinAccount: PublicKey;
  let adminCollateralAccount: PublicKey;
  let user1StablecoinAccount: PublicKey;
  let user1CollateralAccount: PublicKey;
  let user2StablecoinAccount: PublicKey;
  let user2CollateralAccount: PublicKey;

  // Program state accounts
  let protocolState: PublicKey;
  let oracleState: PublicKey;
  let feesState: PublicKey;

  // User trove accounts
  let user1Trove: PublicKey;
  let user2Trove: PublicKey;

  // User stake accounts
  let user1Stake: PublicKey;
  let user2Stake: PublicKey;

  // PDA accounts
  let user1DebtAmountPDA: PublicKey;
  let user1LiquidityThresholdPDA: PublicKey;
  let user1CollateralAmountPDA: PublicKey;
  let user1NodePDA: PublicKey;
  let protocolCollateralAccountPDA: PublicKey;
  let totalCollateralAmountPDA: PublicKey;
  let sortedTrovesStatePDA: PublicKey;
  let protocolStablecoinAccountPDA: PublicKey;
  let stabilityPoolTokenAccount: PublicKey;
  let feeAddress1TokenAccount: PublicKey;
  let feeAddress2TokenAccount: PublicKey;

  before(async () => {
    // Airdrop SOL to admin
    const signature = await provider.connection.requestAirdrop(admin.publicKey, 1000000000);
    await provider.connection.confirmTransaction(signature);

    // Create token mints
    stablecoinMint = await createMint(provider.connection, admin, admin.publicKey, null, 6);
    collateralMint = await createMint(provider.connection, admin, admin.publicKey, null, 6);

    // Create token accounts
    adminStablecoinAccount = await getAssociatedTokenAddress(stablecoinMint, admin.publicKey);
    adminCollateralAccount = await getAssociatedTokenAddress(collateralMint, admin.publicKey);
    user1StablecoinAccount = await getAssociatedTokenAddress(stablecoinMint, user1.publicKey);
    user1CollateralAccount = await getAssociatedTokenAddress(collateralMint, user1.publicKey);
    user2StablecoinAccount = await getAssociatedTokenAddress(stablecoinMint, user2.publicKey);
    user2CollateralAccount = await getAssociatedTokenAddress(collateralMint, user2.publicKey);

    // Create token accounts
    await createAssociatedTokenAccount(provider.connection, admin, stablecoinMint, admin.publicKey);
    await createAssociatedTokenAccount(provider.connection, admin, collateralMint, admin.publicKey);
    await createAssociatedTokenAccount(provider.connection, admin, stablecoinMint, user1.publicKey);
    await createAssociatedTokenAccount(provider.connection, admin, collateralMint, user1.publicKey);
    await createAssociatedTokenAccount(provider.connection, admin, stablecoinMint, user2.publicKey);
    await createAssociatedTokenAccount(provider.connection, admin, collateralMint, user2.publicKey);

    // Mint initial tokens
    await mintTo(provider.connection, admin, stablecoinMint, adminStablecoinAccount, admin, 1000000000);
    await mintTo(provider.connection, admin, collateralMint, adminCollateralAccount, admin, 1000000000);

    // Transfer tokens to users
    await transfer(provider.connection, admin, adminCollateralAccount, user1CollateralAccount, admin, 100000000);
    await transfer(provider.connection, admin, adminCollateralAccount, user2CollateralAccount, admin, 100000000);

    // Derive state PDAs
    const [protocolStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      protocolProgram.programId
    );
    const [oracleStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      oracleProgram.programId
    );
    const [feesStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      feesProgram.programId
    );

    protocolState = protocolStatePda;
    oracleState = oracleStatePda;
    feesState = feesStatePda;

    // Derive user trove PDAs
    const [user1TrovePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("trove"), user1.publicKey.toBuffer()],
      protocolProgram.programId
    );
    const [user2TrovePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("trove"), user2.publicKey.toBuffer()],
      protocolProgram.programId
    );

    user1Trove = user1TrovePda;
    user2Trove = user2TrovePda;

    // Derive user stake PDAs
    const [user1StakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), user1.publicKey.toBuffer()],
      protocolProgram.programId
    );
    const [user2StakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), user2.publicKey.toBuffer()],
      protocolProgram.programId
    );

    user1Stake = user1StakePda;
    user2Stake = user2StakePda;

    // Derive additional PDAs for openTrove
    const [user1DebtAmountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_debt_amount"), user1.publicKey.toBuffer()],
      protocolProgram.programId
    );
    const [user1LiquidityThresholdPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_threshold"), user1.publicKey.toBuffer()],
      protocolProgram.programId
    );
    const [user1CollateralAmountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_collateral_amount"), user1.publicKey.toBuffer(), Buffer.from("SOL")],
      protocolProgram.programId
    );
    const [user1NodePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("node"), user1.publicKey.toBuffer()],
      protocolProgram.programId
    );
    const [protocolCollateralAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("protocol_collateral_vault"), Buffer.from("SOL")],
      protocolProgram.programId
    );
    const [totalCollateralAmountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
      protocolProgram.programId
    );
    const [sortedTrovesStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("sorted_troves_state")],
      protocolProgram.programId
    );
    const [protocolStablecoinAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("protocol_stablecoin_vault")],
      protocolProgram.programId
    );

    user1DebtAmountPDA = user1DebtAmountPda;
    user1LiquidityThresholdPDA = user1LiquidityThresholdPda;
    user1CollateralAmountPDA = user1CollateralAmountPda;
    user1NodePDA = user1NodePda;
    protocolCollateralAccountPDA = protocolCollateralAccountPda;
    totalCollateralAmountPDA = totalCollateralAmountPda;
    sortedTrovesStatePDA = sortedTrovesStatePda;
    protocolStablecoinAccountPDA = protocolStablecoinAccountPda;

    // Get associated token addresses for fee addresses and stability pool
    const feeAddress1 = Keypair.generate();
    const feeAddress2 = Keypair.generate();

    // Airdrop SOL to fee addresses
    await provider.connection.requestAirdrop(feeAddress1.publicKey, 1 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(feeAddress2.publicKey, 1 * LAMPORTS_PER_SOL);

    // Wait for airdrops to complete
    await new Promise(resolve => setTimeout(resolve, 1000));

    stabilityPoolTokenAccount = await getAssociatedTokenAddress(
      stablecoinMint,
      protocolStablecoinAccountPDA,
      true // allowOwnerOffCurve
    );
    feeAddress1TokenAccount = await getAssociatedTokenAddress(
      stablecoinMint,
      feeAddress1.publicKey
    );
    feeAddress2TokenAccount = await getAssociatedTokenAddress(
      stablecoinMint,
      feeAddress2.publicKey
    );

    // Initialize programs
    try {
      await oracleProgram.methods
        .initialize()
        .accounts({
          state: oracleState,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([admin])
        .rpc();
    } catch (e) {
      console.log("Oracle already initialized");
    }

    try {
      await feesProgram.methods
        .initialize()
        .accounts({
          state: feesState,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([admin])
        .rpc();
    } catch (e) {
      console.log("Fees already initialized");
    }

    try {
      await protocolProgram.methods
        .initialize()
        .accounts({
          state: protocolState,
          admin: admin.publicKey,
          oracleProgram: oracleProgram.programId,
          oracleState: oracleState,
          feesProgram: feesProgram.programId,
          feesState: feesState,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([admin])
        .rpc();
    } catch (e) {
      console.log("Protocol already initialized");
    }
  });

  describe("Core Protocol Operations", () => {
    it("Should open a trove successfully", async () => {
      const collateralAmount = 10000000; // 10 tokens
      const loanAmount = 5000000; // 5 stablecoins
      const collateralDenom = "SOL";

      try {
        await protocolProgram.methods
          .openTrove({
            collateralAmount: new anchor.BN(collateralAmount),
            loanAmount: new anchor.BN(loanAmount),
            collateralDenom: collateralDenom,
          })
          .accounts({
            user: user1.publicKey,
            userDebtAmount: user1DebtAmountPDA,
            liquidityThreshold: user1LiquidityThresholdPDA,
            userCollateralAmount: user1CollateralAmountPDA,
            userCollateralAccount: user1CollateralAccount,
            protocolCollateralAccount: protocolCollateralAccountPDA,
            totalCollateralAmount: totalCollateralAmountPDA,
            sortedTrovesState: sortedTrovesStatePDA,
            node: user1NodePDA,
            state: protocolState,
            userStablecoinAccount: user1StablecoinAccount,
            protocolStablecoinAccount: protocolStablecoinAccountPDA,
            stableCoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            pythPriceAccount: PYTH_ORACLE_ADDRESS,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddress1TokenAccount,
            feeAddress2TokenAccount: feeAddress2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Trove opened successfully");
      } catch (error) {
        console.log("❌ Trove opening failed:", error);
        throw error;
      }
    });

    it("Should add collateral to existing trove", async () => {
      const additionalCollateral = 5000000; // 5 more tokens

      try {
        await protocolProgram.methods
          .addCollateral({
            collateralAmount: new anchor.BN(additionalCollateral),
            collateralDenom: "SOL",
          })
          .accounts({
            user: user1.publicKey,
            userDebtAmount: user1DebtAmountPDA,
            userCollateralAmount: user1CollateralAmountPDA,
            liquidityThreshold: user1LiquidityThresholdPDA,
            state: protocolState,
            userCollateralAccount: user1CollateralAccount,
            protocolCollateralAccount: protocolCollateralAccountPDA,
            totalCollateralAmount: totalCollateralAmountPDA,
            sortedTrovesState: sortedTrovesStatePDA,
            node: user1NodePDA,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            pythPriceAccount: PYTH_ORACLE_ADDRESS,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Collateral added successfully");
      } catch (error) {
        console.log("❌ Adding collateral failed:", error);
        throw error;
      }
    });

    it("Should borrow additional loan from trove", async () => {
      const additionalLoan = 2000000; // 2 more stablecoins

      try {
        await protocolProgram.methods
          .borrowLoan({
            loanAmount: new anchor.BN(additionalLoan),
            collateralDenom: "SOL",
          })
          .accounts({
            user: user1.publicKey,
            userDebtAmount: user1DebtAmountPDA,
            liquidityThreshold: user1LiquidityThresholdPDA,
            userCollateralAmount: user1CollateralAmountPDA,
            userCollateralAccount: user1CollateralAccount,
            protocolCollateralAccount: protocolCollateralAccountPDA,
            totalCollateralAmount: totalCollateralAmountPDA,
            sortedTrovesState: sortedTrovesStatePDA,
            node: user1NodePDA,
            state: protocolState,
            userStablecoinAccount: user1StablecoinAccount,
            protocolStablecoinAccount: protocolStablecoinAccountPDA,
            stableCoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            pythPriceAccount: PYTH_ORACLE_ADDRESS,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddress1TokenAccount,
            feeAddress2TokenAccount: feeAddress2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Additional loan borrowed successfully");
      } catch (error) {
        console.log("❌ Borrowing additional loan failed:", error);
        throw error;
      }
    });

    it("Should stake stablecoins in stability pool", async () => {
      const stakeAmount = 3000000; // 3 stablecoins

      try {
        await protocolProgram.methods
          .stake({
            amount: new anchor.BN(stakeAmount),
          })
          .accounts({
            user: user1.publicKey,
            userStakeAmount: user1Stake,
            state: protocolState,
            userStablecoinAccount: user1StablecoinAccount,
            protocolStablecoinAccount: protocolStablecoinAccountPDA,
            stableCoinMint: stablecoinMint,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddress1TokenAccount,
            feeAddress2TokenAccount: feeAddress2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Stablecoins staked successfully");
      } catch (error) {
        console.log("❌ Staking failed:", error);
        throw error;
      }
    });

    it("Should open a second trove for user2", async () => {
      const collateralAmount = 8000000; // 8 tokens
      const loanAmount = 3000000; // 3 stablecoins
      const collateralDenom = "SOL";

      // Derive user2 PDAs
      const [user2DebtAmountPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), user2.publicKey.toBuffer()],
        protocolProgram.programId
      );
      const [user2LiquidityThresholdPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("liquidity_threshold"), user2.publicKey.toBuffer()],
        protocolProgram.programId
      );
      const [user2CollateralAmountPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_collateral_amount"), user2.publicKey.toBuffer(), Buffer.from("SOL")],
        protocolProgram.programId
      );
      const [user2NodePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("node"), user2.publicKey.toBuffer()],
        protocolProgram.programId
      );

      try {
        await protocolProgram.methods
          .openTrove({
            collateralAmount: new anchor.BN(collateralAmount),
            loanAmount: new anchor.BN(loanAmount),
            collateralDenom: collateralDenom,
          })
          .accounts({
            user: user2.publicKey,
            userDebtAmount: user2DebtAmountPda,
            liquidityThreshold: user2LiquidityThresholdPda,
            userCollateralAmount: user2CollateralAmountPda,
            userCollateralAccount: user2CollateralAccount,
            protocolCollateralAccount: protocolCollateralAccountPDA,
            totalCollateralAmount: totalCollateralAmountPDA,
            sortedTrovesState: sortedTrovesStatePDA,
            node: user2NodePda,
            state: protocolState,
            userStablecoinAccount: user2StablecoinAccount,
            protocolStablecoinAccount: protocolStablecoinAccountPDA,
            stableCoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            pythPriceAccount: PYTH_ORACLE_ADDRESS,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddress1TokenAccount,
            feeAddress2TokenAccount: feeAddress2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user2])
          .rpc();

        console.log("✅ Second trove opened successfully");
      } catch (error) {
        console.log("❌ Second trove opening failed:", error);
        throw error;
      }
    });

    it("Should repay loan partially", async () => {
      const repayAmount = 2000000; // 2 stablecoins

      try {
        await protocolProgram.methods
          .repayLoan({
            repayAmount: new anchor.BN(repayAmount),
            collateralDenom: "SOL",
          })
          .accounts({
            user: user1.publicKey,
            userDebtAmount: user1DebtAmountPDA,
            liquidityThreshold: user1LiquidityThresholdPDA,
            userCollateralAmount: user1CollateralAmountPDA,
            userCollateralAccount: user1CollateralAccount,
            protocolCollateralAccount: protocolCollateralAccountPDA,
            totalCollateralAmount: totalCollateralAmountPDA,
            sortedTrovesState: sortedTrovesStatePDA,
            node: user1NodePDA,
            state: protocolState,
            userStablecoinAccount: user1StablecoinAccount,
            protocolStablecoinAccount: protocolStablecoinAccountPDA,
            stableCoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            pythPriceAccount: PYTH_ORACLE_ADDRESS,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddress1TokenAccount,
            feeAddress2TokenAccount: feeAddress2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Loan partially repaid successfully");
      } catch (error) {
        console.log("❌ Loan repayment failed:", error);
        throw error;
      }
    });

    it("Should remove collateral from trove", async () => {
      const removeAmount = 3000000; // 3 tokens

      try {
        await protocolProgram.methods
          .removeCollateral({
            collateralAmount: new anchor.BN(removeAmount),
            collateralDenom: "SOL",
          })
          .accounts({
            user: user1.publicKey,
            userDebtAmount: user1DebtAmountPDA,
            userCollateralAmount: user1CollateralAmountPDA,
            liquidityThreshold: user1LiquidityThresholdPDA,
            state: protocolState,
            userCollateralAccount: user1CollateralAccount,
            protocolCollateralAccount: protocolCollateralAccountPDA,
            totalCollateralAmount: totalCollateralAmountPDA,
            sortedTrovesState: sortedTrovesStatePDA,
            node: user1NodePDA,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            pythPriceAccount: PYTH_ORACLE_ADDRESS,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Collateral removed successfully");
      } catch (error) {
        console.log("❌ Removing collateral failed:", error);
        throw error;
      }
    });

    it("Should unstake stablecoins from stability pool", async () => {
      const unstakeAmount = 1500000; // 1.5 stablecoins

      try {
        await protocolProgram.methods
          .unstake({
            amount: new anchor.BN(unstakeAmount),
          })
          .accounts({
            user: user1.publicKey,
            userStakeAmount: user1Stake,
            state: protocolState,
            userStablecoinAccount: user1StablecoinAccount,
            protocolStablecoinAccount: protocolStablecoinAccountPDA,
            stableCoinMint: stablecoinMint,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddress1TokenAccount,
            feeAddress2TokenAccount: feeAddress2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Stablecoins unstaked successfully");
      } catch (error) {
        console.log("❌ Unstaking failed:", error);
        throw error;
      }
    });
  });

  describe("Protocol State Verification", () => {
    it("Should verify protocol state after operations", async () => {
      try {
        const stateAccount = await protocolProgram.account.stateAccount.fetch(protocolState);
        console.log("📊 Protocol State:");
        console.log("- Total Debt Amount:", stateAccount.totalDebtAmount.toString());
        console.log("- Total Collateral Amount:", stateAccount.totalCollateralAmount.toString());
        console.log("- Total Stake Amount:", stateAccount.totalStakeAmount.toString());
        console.log("- Minimum Collateral Ratio:", stateAccount.minimumCollateralRatio.toString());
        console.log("- Protocol Fee:", stateAccount.protocolFee.toString());

        assert(stateAccount.totalDebtAmount.gt(new anchor.BN(0)), "Total debt should be greater than 0");
        assert(stateAccount.totalCollateralAmount.gt(new anchor.BN(0)), "Total collateral should be greater than 0");
        assert(stateAccount.totalStakeAmount.gt(new anchor.BN(0)), "Total stake should be greater than 0");

        console.log("✅ Protocol state verification passed");
      } catch (error) {
        console.log("❌ Protocol state verification failed:", error);
        throw error;
      }
    });

    it("Should verify user trove state", async () => {
      try {
        const troveAccount = await protocolProgram.account.troveAccount.fetch(user1Trove);
        console.log("📊 User1 Trove State:");
        console.log("- Owner:", troveAccount.owner.toString());
        console.log("- Debt Amount:", troveAccount.debtAmount.toString());
        console.log("- Collateral Amount:", troveAccount.collateralAmount.toString());
        console.log("- Collateral Ratio:", troveAccount.collateralRatio.toString());
        console.log("- Is Active:", troveAccount.isActive);

        assert(troveAccount.owner.equals(user1.publicKey), "Trove owner should match user1");
        assert(troveAccount.isActive, "Trove should be active");
        assert(troveAccount.debtAmount.gt(new anchor.BN(0)), "Debt amount should be greater than 0");
        assert(troveAccount.collateralAmount.gt(new anchor.BN(0)), "Collateral amount should be greater than 0");

        console.log("✅ User trove verification passed");
      } catch (error) {
        console.log("❌ User trove verification failed:", error);
        throw error;
      }
    });

    it("Should verify user stake state", async () => {
      try {
        const stakeAccount = await protocolProgram.account.stakeAccount.fetch(user1Stake);
        console.log("📊 User1 Stake State:");
        console.log("- Owner:", stakeAccount.owner.toString());
        console.log("- Amount:", stakeAccount.amount.toString());
        console.log("- Total Stake At Time:", stakeAccount.totalStakeAtTime.toString());
        console.log("- Percentage:", stakeAccount.percentage.toString());

        assert(stakeAccount.owner.equals(user1.publicKey), "Stake owner should match user1");
        assert(stakeAccount.amount.gt(new anchor.BN(0)), "Stake amount should be greater than 0");

        console.log("✅ User stake verification passed");
      } catch (error) {
        console.log("❌ User stake verification failed:", error);
        throw error;
      }
    });
  });
});

// Helper functions removed - using SPL Token library directly
