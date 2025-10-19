import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert, expect } from "chai";

describe("Protocol Contract - Trove Management Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  const admin = provider.wallet;
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();

  const PYTH_ORACLE_ADDRESS = new PublicKey("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");

  let stablecoinMint: PublicKey;
  let collateralMint: PublicKey;
  let protocolState: PublicKey;
  let oracleState: PublicKey;
  let feeState: PublicKey;
  let protocolVault: PublicKey;
  let user1CollateralAccount: PublicKey;
  let user1StablecoinAccount: PublicKey;
  let user2CollateralAccount: PublicKey;

  before(async () => {
    console.log("\n🚀 Setting up Trove Management Tests...");

    // Transfer SOL for transaction fees and account creation
    const transferAmount = 10000000; // 0.01 SOL in lamports
    
    const user1Tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: admin.publicKey,
        toPubkey: user1.publicKey,
        lamports: transferAmount,
      })
    );
    await provider.sendAndConfirm(user1Tx, [admin.payer]);

    const user2Tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: admin.publicKey,
        toPubkey: user2.publicKey,
        lamports: transferAmount,
      })
    );
    await provider.sendAndConfirm(user2Tx, [admin.payer]);

    // Create mints
    stablecoinMint = await createMint(provider.connection, admin.payer, admin.publicKey, null, 18);
    collateralMint = await createMint(provider.connection, admin.payer, admin.publicKey, null, 9);

    // Create token accounts
    user1CollateralAccount = await createAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      collateralMint,
      user1.publicKey
    );
    user1StablecoinAccount = await createAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      stablecoinMint,
      user1.publicKey
    );
    user2CollateralAccount = await createAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      collateralMint,
      user2.publicKey
    );

    // Mint collateral to users
    await mintTo(
      provider.connection,
      admin.payer,
      collateralMint,
      user1CollateralAccount,
      admin.publicKey,
      100_000_000_000
    );
    await mintTo(
      provider.connection,
      admin.payer,
      collateralMint,
      user2CollateralAccount,
      admin.publicKey,
      100_000_000_000
    );

    // Initialize oracle
    const oracleStateKeypair = Keypair.generate();
    oracleState = oracleStateKeypair.publicKey;

    await oracleProgram.methods
      .initialize({ oracleAddress: PYTH_ORACLE_ADDRESS })
      .accounts({
        state: oracleState,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      })
      .signers([oracleStateKeypair])
      .rpc();

    // Initialize fees
    const feeStateKeypair = Keypair.generate();
    feeState = feeStateKeypair.publicKey;

    await feesProgram.methods
      .initialize()
      .accounts({
        state: feeState,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([feeStateKeypair])
      .rpc();

    // Initialize protocol
    const protocolStateKeypair = Keypair.generate();
    protocolState = protocolStateKeypair.publicKey;

    await protocolProgram.methods
      .initialize({
        stableCoinCodeId: new anchor.BN(1),
        oracleHelperAddr: oracleProgram.programId,
        oracleStateAddr: oracleState,
        feeDistributorAddr: feesProgram.programId,
        feeStateAddr: feeState,
      })
      .accounts({
        state: protocolState,
        admin: admin.publicKey,
        stableCoinMint: stablecoinMint,
        systemProgram: SystemProgram.programId,
      })
      .signers([protocolStateKeypair])
      .rpc();

    // Derive protocol vault
    [protocolVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("protocol_vault"), collateralMint.toBuffer()],
      protocolProgram.programId
    );

    console.log("✅ Setup complete");
    console.log("  User1:", user1.publicKey.toString());
    console.log("  Protocol State:", protocolState.toString());
    console.log("  Collateral Mint:", collateralMint.toString());
    console.log("  Stablecoin Mint:", stablecoinMint.toString());
  });

  describe("Test 2.1: Open Trove with Valid Collateral", () => {
    it("Should successfully open trove with sufficient collateral", async () => {
      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), user1.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          user1.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const collateralAmount = new BN(10_000_000_000); // 10 SOL
      const loanAmount = new BN(1_000_000_000_000_000_000); // 1 aUSD

      console.log("📋 Opening trove...");
      console.log("  Collateral:", collateralAmount.toString(), "lamports (10 SOL)");
      console.log("  Loan:", loanAmount.toString(), "base units (1 aUSD)");

      const tx = await protocolProgram.methods
        .openTrove({
          collateralAmount,
          loanAmount,
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebtAmount: userDebtPda,
          userCollateralAmount: userCollateralPda,
          totalCollateralAmount: totalCollateralPda,
          user: user1.publicKey,
          userCollateralAccount: user1CollateralAccount,
          protocolCollateralAccount: protocolVault,
          userStablecoinAccount: user1StablecoinAccount,
          protocolStablecoinAccount: protocolVault,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          liquidityThreshold: userDebtPda, // Use same PDA for now
          node: userDebtPda, // Use same PDA for now
          sortedTrovesState: userDebtPda, // Use same PDA for now
          oracleProgram: oracleProgram.programId,
          oracleState: oracleState,
          pythPriceAccount: PYTH_ORACLE_ADDRESS,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          feesProgram: feesProgram.programId,
          feesState: feeState,
          stabilityPoolTokenAccount: user1StablecoinAccount, // Use user account for now
          feeAddress1TokenAccount: user1StablecoinAccount, // Use user account for now
          feeAddress2TokenAccount: user1StablecoinAccount, // Use user account for now
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      console.log("✅ Trove opened. TX:", tx);

      const userDebt = await protocolProgram.account.userDebtAmount.fetch(userDebtPda);
      const userCollateral = await protocolProgram.account.userCollateralAmount.fetch(
        userCollateralPda
      );

      assert.equal(userDebt.amount.toString(), loanAmount.toString());
      assert.equal(userCollateral.amount.toString(), collateralAmount.toString());

      console.log("✅ Trove state verified");
    });
  });

  describe("Test 2.2: Reject Duplicate Trove Opening", () => {
    it("Should fail when user tries to open second trove", async () => {
      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), user2.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          user2.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const user2StablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        user2.publicKey
      );

      // First trove opens successfully
      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(10_000_000_000),
          loanAmount: new BN(1_000_000_000_000_000_000),
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: user2.publicKey,
          userCollateralAccount: user2CollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: user2StablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      console.log("🔒 Attempting to open duplicate trove...");

      try {
        await protocolProgram.methods
          .openTrove({
            collateralAmount: new BN(5_000_000_000),
            loanAmount: new BN(500_000_000_000_000_000),
            denom: "SOL",
          })
          .accounts({
            state: protocolState,
            userDebt: userDebtPda,
            userCollateral: userCollateralPda,
            totalCollateral: totalCollateralPda,
            user: user2.publicKey,
            userCollateralAccount: user2CollateralAccount,
            protocolVault: protocolVault,
            userStablecoinAccount: user2StablecoinAccount,
            stableCoinMint: stablecoinMint,
            collateralMint: collateralMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user2])
          .rpc();

        assert.fail("Should have rejected duplicate trove");
      } catch (error: any) {
        console.log("✅ Duplicate trove correctly rejected");
        expect(error.message).to.include("already in use");
      }
    });
  });

  describe("Test 2.3: Add Collateral to Existing Trove", () => {
    it("Should successfully add collateral", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      // Open trove
      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(10_000_000_000),
          loanAmount: new BN(1_000_000_000_000_000_000),
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const initialCollateral = await protocolProgram.account.userCollateralAmount.fetch(
        userCollateralPda
      );

      console.log("📋 Adding collateral...");
      console.log("  Initial:", initialCollateral.amount.toString());

      // Add collateral
      await protocolProgram.methods
        .addCollateral({
          collateralAmount: new BN(5_000_000_000),
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const finalCollateral = await protocolProgram.account.userCollateralAmount.fetch(
        userCollateralPda
      );

      console.log("  Final:", finalCollateral.amount.toString());

      const expected = initialCollateral.amount.add(new BN(5_000_000_000));
      assert.equal(finalCollateral.amount.toString(), expected.toString());

      console.log("✅ Collateral added successfully");
    });
  });

  describe("Test 2.4: Borrow Loan from Trove", () => {
    it("Should successfully borrow additional loan", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      // Open trove with high collateral
      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(20_000_000_000),
          loanAmount: new BN(1_000_000_000_000_000_000),
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const initialDebt = await protocolProgram.account.userDebtAmount.fetch(userDebtPda);
      console.log("📋 Borrowing additional loan...");
      console.log("  Initial debt:", initialDebt.amount.toString());

      // Borrow more
      const additionalLoan = new BN(500_000_000_000_000_000);
      await protocolProgram.methods
        .borrowLoan({
          loanAmount: additionalLoan,
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          user: testUser.publicKey,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const finalDebt = await protocolProgram.account.userDebtAmount.fetch(userDebtPda);
      console.log("  Final debt:", finalDebt.amount.toString());

      const expected = initialDebt.amount.add(additionalLoan);
      assert.equal(finalDebt.amount.toString(), expected.toString());

      console.log("✅ Loan borrowed successfully");
    });
  });

  describe("Test 2.5: Repay Loan Partially", () => {
    it("Should successfully repay part of the loan", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      // Open trove
      const loanAmount = new BN(2_000_000_000_000_000_000);
      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(20_000_000_000),
          loanAmount,
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const initialDebt = await protocolProgram.account.userDebtAmount.fetch(userDebtPda);
      console.log("📋 Repaying loan partially...");
      console.log("  Initial debt:", initialDebt.amount.toString());

      // Repay half
      const repayAmount = new BN(1_000_000_000_000_000_000);
      await protocolProgram.methods
        .repayLoan({
          repayAmount,
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          user: testUser.publicKey,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const finalDebt = await protocolProgram.account.userDebtAmount.fetch(userDebtPda);
      console.log("  Final debt:", finalDebt.amount.toString());

      const expected = initialDebt.amount.sub(repayAmount);
      assert.equal(finalDebt.amount.toString(), expected.toString());

      console.log("✅ Loan repaid partially");
    });
  });

  describe("Test 2.6: Repay Loan Fully", () => {
    it("Should successfully repay all debt", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      // Open trove
      const loanAmount = new BN(1_000_000_000_000_000_000);
      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(15_000_000_000),
          loanAmount,
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      console.log("📋 Repaying full loan...");

      // Repay all
      await protocolProgram.methods
        .repayLoan({
          repayAmount: loanAmount,
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          user: testUser.publicKey,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const finalDebt = await protocolProgram.account.userDebtAmount.fetch(userDebtPda);
      assert.equal(finalDebt.amount.toString(), "0");

      console.log("✅ Loan fully repaid");
    });
  });

  describe("Test 2.7: Close Trove", () => {
    it("Should successfully close trove after full repayment", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      const loanAmount = new BN(1_000_000_000_000_000_000);
      const collateralAmount = new BN(15_000_000_000);

      // Open trove
      await protocolProgram.methods
        .openTrove({
          collateralAmount,
          loanAmount,
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      console.log("📋 Closing trove...");

      // Close trove
      await protocolProgram.methods
        .closeTrove({
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          userStablecoinAccount: testStablecoinAccount,
          protocolVault: protocolVault,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      // Verify accounts are closed
      const debtAccount = await provider.connection.getAccountInfo(userDebtPda);
      const collateralAccount = await provider.connection.getAccountInfo(userCollateralPda);

      assert.isNull(debtAccount, "Debt account should be closed");
      assert.isNull(collateralAccount, "Collateral account should be closed");

      console.log("✅ Trove closed successfully");
    });
  });

  describe("Test 2.8: Remove Collateral (Maintaining MCR)", () => {
    it("Should successfully remove collateral while maintaining MCR", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      // Open trove with excess collateral
      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(30_000_000_000),
          loanAmount: new BN(1_000_000_000_000_000_000),
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const initialCollateral = await protocolProgram.account.userCollateralAmount.fetch(
        userCollateralPda
      );

      console.log("📋 Removing collateral...");
      console.log("  Initial:", initialCollateral.amount.toString());

      // Remove some collateral (maintaining MCR)
      await protocolProgram.methods
        .removeCollateral({
          collateralAmount: new BN(5_000_000_000),
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const finalCollateral = await protocolProgram.account.userCollateralAmount.fetch(
        userCollateralPda
      );

      console.log("  Final:", finalCollateral.amount.toString());

      const expected = initialCollateral.amount.sub(new BN(5_000_000_000));
      assert.equal(finalCollateral.amount.toString(), expected.toString());

      console.log("✅ Collateral removed successfully");
    });
  });

  describe("Test 2.9: Reject Collateral Removal Below MCR", () => {
    it("Should fail when removing collateral would violate MCR", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      // Open trove with minimal collateral
      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(12_000_000_000),
          loanAmount: new BN(1_000_000_000_000_000_000),
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      console.log("🔒 Attempting to remove collateral below MCR...");

      try {
        await protocolProgram.methods
          .removeCollateral({
            collateralAmount: new BN(10_000_000_000),
            denom: "SOL",
          })
          .accounts({
            state: protocolState,
            userDebt: userDebtPda,
            userCollateral: userCollateralPda,
            totalCollateral: totalCollateralPda,
            user: testUser.publicKey,
            userCollateralAccount: testCollateralAccount,
            protocolVault: protocolVault,
            collateralMint: collateralMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([testUser])
          .rpc();

        assert.fail("Should have rejected removal below MCR");
      } catch (error: any) {
        console.log("✅ Removal below MCR correctly rejected");
        expect(error.message).to.include("InvalidCollateralRatio");
      }
    });
  });

  describe("Test 2.10: Reject Borrow Below Minimum Loan Amount", () => {
    it("Should fail when borrowing below minimum loan amount", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      console.log("🔒 Attempting to open trove with loan below minimum...");

      try {
        await protocolProgram.methods
          .openTrove({
            collateralAmount: new BN(10_000_000_000),
            loanAmount: new BN(100_000_000_000_000_000), // 0.1 aUSD (below 1 aUSD minimum)
            denom: "SOL",
          })
          .accounts({
            state: protocolState,
            userDebt: userDebtPda,
            userCollateral: userCollateralPda,
            totalCollateral: totalCollateralPda,
            user: testUser.publicKey,
            userCollateralAccount: testCollateralAccount,
            protocolVault: protocolVault,
            userStablecoinAccount: testStablecoinAccount,
            stableCoinMint: stablecoinMint,
            collateralMint: collateralMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([testUser])
          .rpc();

        assert.fail("Should have rejected loan below minimum");
      } catch (error: any) {
        console.log("✅ Loan below minimum correctly rejected");
        expect(error.message).to.include("LoanAmountBelowMinimum");
      }
    });
  });

  describe("Test 2.11: Reject Close Trove with Outstanding Debt", () => {
    it("Should fail when trying to close trove with debt", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("SOL"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("SOL")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      // Open trove
      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(15_000_000_000),
          loanAmount: new BN(1_000_000_000_000_000_000),
          denom: "SOL",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      console.log("🔒 Attempting to close trove with outstanding debt...");

      try {
        await protocolProgram.methods
          .closeTrove({
            denom: "SOL",
          })
          .accounts({
            state: protocolState,
            userDebt: userDebtPda,
            userCollateral: userCollateralPda,
            totalCollateral: totalCollateralPda,
            user: testUser.publicKey,
            userCollateralAccount: testCollateralAccount,
            userStablecoinAccount: testStablecoinAccount,
            protocolVault: protocolVault,
            stableCoinMint: stablecoinMint,
            collateralMint: collateralMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([testUser])
          .rpc();

        assert.fail("Should have rejected closing trove with debt");
      } catch (error: any) {
        console.log("✅ Close with outstanding debt correctly rejected");
        expect(error.message).to.include("OutstandingDebt");
      }
    });
  });

  describe("Test 2.12: Open Trove with Multiple Collateral Types", () => {
    it("Should support multiple collateral denominations", async () => {
      const testUser = Keypair.generate();
      const transferAmount = 10000000; // 0.01 SOL in lamports
      const testUserTx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: admin.publicKey,
          toPubkey: testUser.publicKey,
          lamports: transferAmount,
        })
      );
      await provider.sendAndConfirm(testUserTx, [admin.payer]);

      const testCollateralAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        collateralMint,
        testUser.publicKey
      );

      await mintTo(
        provider.connection,
        admin.payer,
        collateralMint,
        testCollateralAccount,
        admin.publicKey,
        50_000_000_000
      );

      const [userDebtPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_debt_amount"), testUser.publicKey.toBuffer()],
        protocolProgram.programId
      );

      const [userCollateralPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("user_collateral_amount"),
          testUser.publicKey.toBuffer(),
          Buffer.from("USDC"),
        ],
        protocolProgram.programId
      );

      const [totalCollateralPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("total_collateral_amount"), Buffer.from("USDC")],
        protocolProgram.programId
      );

      const testStablecoinAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        stablecoinMint,
        testUser.publicKey
      );

      console.log("📋 Opening trove with USDC collateral...");

      await protocolProgram.methods
        .openTrove({
          collateralAmount: new BN(10_000_000_000),
          loanAmount: new BN(1_000_000_000_000_000_000),
          denom: "USDC",
        })
        .accounts({
          state: protocolState,
          userDebt: userDebtPda,
          userCollateral: userCollateralPda,
          totalCollateral: totalCollateralPda,
          user: testUser.publicKey,
          userCollateralAccount: testCollateralAccount,
          protocolVault: protocolVault,
          userStablecoinAccount: testStablecoinAccount,
          stableCoinMint: stablecoinMint,
          collateralMint: collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      const userCollateral = await protocolProgram.account.userCollateralAmount.fetch(
        userCollateralPda
      );

      assert.equal(userCollateral.denom, "USDC");
      assert.equal(userCollateral.amount.toString(), "10000000000");

      console.log("✅ Multiple collateral types supported");
    });
  });
});
