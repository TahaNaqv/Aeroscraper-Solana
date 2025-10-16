import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { createMint } from "@solana/spl-token";
import { assert, expect } from "chai";

describe("Protocol Contract - Initialization Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  const PYTH_ORACLE_ADDRESS = new PublicKey("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");

  let stablecoinMint: PublicKey;
  let oracleState: PublicKey;
  let feeState: PublicKey;

  before(async () => {
    console.log("\n🚀 Setting up Protocol Initialization Tests...");
    console.log("  Admin:", provider.wallet.publicKey.toString());

    // Create stablecoin mint
    stablecoinMint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      18
    );

    // Initialize oracle program
    const oracleStateKeypair = Keypair.generate();
    oracleState = oracleStateKeypair.publicKey;

    await oracleProgram.methods
      .initialize({
        oracleAddress: PYTH_ORACLE_ADDRESS,
      })
      .accounts({
        state: oracleState,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      })
      .signers([oracleStateKeypair])
      .rpc();

    // Initialize fees program
    const feeStateKeypair = Keypair.generate();
    feeState = feeStateKeypair.publicKey;

    await feesProgram.methods
      .initialize()
      .accounts({
        state: feeState,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([feeStateKeypair])
      .rpc();

    console.log("✅ Oracle and Fees programs initialized");
    console.log("  Oracle State:", oracleState.toString());
    console.log("  Fee State:", feeState.toString());
    console.log("  Stablecoin Mint:", stablecoinMint.toString());
  });

  describe("Test 1.1: Initialize Protocol Successfully", () => {
    it("Should initialize protocol with correct initial state", async () => {
      const protocolStateKeypair = Keypair.generate();
      
      console.log("📋 Initializing protocol...");
      console.log("  State Account:", protocolStateKeypair.publicKey.toString());

      const tx = await protocolProgram.methods
        .initialize({
          stableCoinCodeId: new anchor.BN(1),
          oracleHelperAddr: oracleProgram.programId,
          oracleStateAddr: oracleState,
          feeDistributorAddr: feesProgram.programId,
          feeStateAddr: feeState,
        })
        .accounts({
          state: protocolStateKeypair.publicKey,
          admin: provider.wallet.publicKey,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolStateKeypair])
        .rpc();

      console.log("✅ Protocol initialized. TX:", tx);

      const state = await protocolProgram.account.stateAccount.fetch(
        protocolStateKeypair.publicKey
      );

      assert.equal(
        state.admin.toString(),
        provider.wallet.publicKey.toString(),
        "Admin should match"
      );
      assert.equal(
        state.stableCoinAddr.toString(),
        stablecoinMint.toString(),
        "Stablecoin mint should match"
      );
      assert.equal(
        state.oracleHelperAddr.toString(),
        oracleProgram.programId.toString(),
        "Oracle program should match"
      );
      assert.equal(
        state.feeDistributorAddr.toString(),
        feesProgram.programId.toString(),
        "Fee distributor should match"
      );

      console.log("✅ All initial state values verified");
    });
  });

  describe("Test 1.2: Prevent Re-initialization", () => {
    it("Should fail when trying to reinitialize same state account", async () => {
      const protocolStateKeypair = Keypair.generate();

      await protocolProgram.methods
        .initialize({
          stableCoinCodeId: new anchor.BN(1),
          oracleHelperAddr: oracleProgram.programId,
          oracleStateAddr: oracleState,
          feeDistributorAddr: feesProgram.programId,
          feeStateAddr: feeState,
        })
        .accounts({
          state: protocolStateKeypair.publicKey,
          admin: provider.wallet.publicKey,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolStateKeypair])
        .rpc();

      console.log("🔒 Attempting to reinitialize same state account...");

      try {
        await protocolProgram.methods
          .initialize({
            stableCoinCodeId: new anchor.BN(1),
            oracleHelperAddr: oracleProgram.programId,
            oracleStateAddr: oracleState,
            feeDistributorAddr: feesProgram.programId,
            feeStateAddr: feeState,
          })
          .accounts({
            state: protocolStateKeypair.publicKey,
            admin: provider.wallet.publicKey,
            stableCoinMint: stablecoinMint,
            systemProgram: SystemProgram.programId,
          })
          .signers([protocolStateKeypair])
          .rpc();

        assert.fail("Should have rejected re-initialization");
      } catch (error: any) {
        console.log("✅ Re-initialization correctly rejected");
        expect(error.message).to.include("already in use");
      }
    });
  });

  describe("Test 1.3: Verify State Properties", () => {
    it("Should have all expected state properties", async () => {
      const protocolStateKeypair = Keypair.generate();

      await protocolProgram.methods
        .initialize({
          stableCoinCodeId: new anchor.BN(1),
          oracleHelperAddr: oracleProgram.programId,
          oracleStateAddr: oracleState,
          feeDistributorAddr: feesProgram.programId,
          feeStateAddr: feeState,
        })
        .accounts({
          state: protocolStateKeypair.publicKey,
          admin: provider.wallet.publicKey,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolStateKeypair])
        .rpc();

      const state = await protocolProgram.account.stateAccount.fetch(
        protocolStateKeypair.publicKey
      );

      expect(state).to.have.property("admin");
      expect(state).to.have.property("oracleHelperAddr");
      expect(state).to.have.property("oracleStateAddr");
      expect(state).to.have.property("feeDistributorAddr");
      expect(state).to.have.property("feeStateAddr");
      expect(state).to.have.property("minimumCollateralRatio");
      expect(state).to.have.property("protocolFee");
      expect(state).to.have.property("stableCoinAddr");
      expect(state).to.have.property("totalDebtAmount");
      expect(state).to.have.property("totalStakeAmount");
      expect(state).to.have.property("pFactor");
      expect(state).to.have.property("epoch");

      console.log("✅ State properties verified:");
      console.log("  admin:", state.admin.toString());
      console.log("  oracle_helper:", state.oracleHelperAddr.toString());
      console.log("  oracle_state:", state.oracleStateAddr.toString());
      console.log("  fee_distributor:", state.feeDistributorAddr.toString());
      console.log("  fee_state:", state.feeStateAddr.toString());
      console.log("  MCR:", state.minimumCollateralRatio);
      console.log("  protocol_fee:", state.protocolFee);
      console.log("  total_debt:", state.totalDebtAmount.toString());
      console.log("  total_stake:", state.totalStakeAmount.toString());
      console.log("  p_factor:", state.pFactor.toString());
      console.log("  epoch:", state.epoch.toString());
    });
  });

  describe("Test 1.4: Validate Default Parameters", () => {
    it("Should initialize with correct default values", async () => {
      const protocolStateKeypair = Keypair.generate();

      await protocolProgram.methods
        .initialize({
          stableCoinMint: stablecoinMint,
          oracleProgram: oracleProgram.programId,
          oracleStateAddr: oracleState,
          feeDistributor: feesProgram.programId,
          feeStateAddr: feeState,
        })
        .accounts({
          state: protocolStateKeypair.publicKey,
          admin: provider.wallet.publicKey,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolStateKeypair])
        .rpc();

      const state = await protocolProgram.account.stateAccount.fetch(
        protocolStateKeypair.publicKey
      );

      assert.equal(state.minimumCollateralRatio, 115, "MCR should be 115%");
      assert.equal(state.protocolFee, 5, "Protocol fee should be 5%");
      assert.equal(state.totalDebtAmount.toString(), "0", "Total debt should be 0");
      assert.equal(state.totalStakeAmount.toString(), "0", "Total stake should be 0");

      console.log("✅ Default parameters verified");
    });
  });

  describe("Test 1.5: Validate P Factor Initialization", () => {
    it("Should initialize P factor to SCALE_FACTOR (10^18)", async () => {
      const protocolStateKeypair = Keypair.generate();

      await protocolProgram.methods
        .initialize({
          stableCoinMint: stablecoinMint,
          oracleProgram: oracleProgram.programId,
          oracleStateAddr: oracleState,
          feeDistributor: feesProgram.programId,
          feeStateAddr: feeState,
        })
        .accounts({
          state: protocolStateKeypair.publicKey,
          admin: provider.wallet.publicKey,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolStateKeypair])
        .rpc();

      const state = await protocolProgram.account.stateAccount.fetch(
        protocolStateKeypair.publicKey
      );

      const SCALE_FACTOR = "1000000000000000000"; // 10^18
      assert.equal(
        state.pFactor.toString(),
        SCALE_FACTOR,
        "P factor should be 10^18"
      );

      console.log("✅ P factor initialized to SCALE_FACTOR");
      console.log("  P factor:", state.pFactor.toString());
    });
  });

  describe("Test 1.6: Validate Epoch Initialization", () => {
    it("Should initialize epoch to 0", async () => {
      const protocolStateKeypair = Keypair.generate();

      await protocolProgram.methods
        .initialize({
          stableCoinMint: stablecoinMint,
          oracleProgram: oracleProgram.programId,
          oracleStateAddr: oracleState,
          feeDistributor: feesProgram.programId,
          feeStateAddr: feeState,
        })
        .accounts({
          state: protocolStateKeypair.publicKey,
          admin: provider.wallet.publicKey,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolStateKeypair])
        .rpc();

      const state = await protocolProgram.account.stateAccount.fetch(
        protocolStateKeypair.publicKey
      );

      assert.equal(state.epoch.toString(), "0", "Epoch should start at 0");

      console.log("✅ Epoch initialized to 0");
    });
  });

  describe("Test 1.7: Validate Oracle and Fee Addresses", () => {
    it("Should correctly store oracle and fee program addresses", async () => {
      const protocolStateKeypair = Keypair.generate();

      await protocolProgram.methods
        .initialize({
          stableCoinCodeId: new anchor.BN(1),
          oracleHelperAddr: oracleProgram.programId,
          oracleStateAddr: oracleState,
          feeDistributorAddr: feesProgram.programId,
          feeStateAddr: feeState,
        })
        .accounts({
          state: protocolStateKeypair.publicKey,
          admin: provider.wallet.publicKey,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolStateKeypair])
        .rpc();

      const state = await protocolProgram.account.stateAccount.fetch(
        protocolStateKeypair.publicKey
      );

      assert.equal(
        state.oracleHelperAddr.toString(),
        oracleProgram.programId.toString(),
        "Oracle program address mismatch"
      );
      assert.equal(
        state.oracleStateAddr.toString(),
        oracleState.toString(),
        "Oracle state address mismatch"
      );
      assert.equal(
        state.feeDistributorAddr.toString(),
        feesProgram.programId.toString(),
        "Fee distributor address mismatch"
      );
      assert.equal(
        state.feeStateAddr.toString(),
        feeState.toString(),
        "Fee state address mismatch"
      );

      console.log("✅ Oracle and fee addresses validated");
    });
  });

  describe("Test 1.8: Validate Stablecoin Mint", () => {
    it("Should correctly store stablecoin mint address", async () => {
      const protocolStateKeypair = Keypair.generate();

      await protocolProgram.methods
        .initialize({
          stableCoinCodeId: new anchor.BN(1),
          oracleHelperAddr: oracleProgram.programId,
          oracleStateAddr: oracleState,
          feeDistributorAddr: feesProgram.programId,
          feeStateAddr: feeState,
        })
        .accounts({
          state: protocolStateKeypair.publicKey,
          admin: provider.wallet.publicKey,
          stableCoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolStateKeypair])
        .rpc();

      const state = await protocolProgram.account.stateAccount.fetch(
        protocolStateKeypair.publicKey
      );

      assert.equal(
        state.stableCoinAddr.toString(),
        stablecoinMint.toString(),
        "Stablecoin mint address mismatch"
      );

      const mintInfo = await provider.connection.getAccountInfo(stablecoinMint);
      assert(mintInfo !== null, "Stablecoin mint should exist");

      console.log("✅ Stablecoin mint validated");
      console.log("  Mint address:", stablecoinMint.toString());
    });
  });
});
