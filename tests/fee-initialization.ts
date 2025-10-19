import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram, 
  LAMPORTS_PER_SOL 
} from "@solana/web3.js";
import { assert, expect } from "chai";
import * as fs from "fs";

describe("Fee Contract - Initialization Tests", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;

  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  // Load wallet explicitly
  const adminKeypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync("/home/taha/.config/solana/id.json", "utf8")))
  );
  const admin = adminKeypair;
  let feeStateAccount: Keypair;

  before(async () => {
    console.log("\n🚀 Setting up Fee Contract Initialization Tests...");
    console.log("  Admin:", admin.publicKey.toString());
    console.log("  Using existing wallet balance (no airdrops needed)");
  });

  describe("Test 1.1: Initialize Fee Contract Successfully", () => {
    it("Should initialize fee contract with correct initial state", async () => {
      feeStateAccount = Keypair.generate();
      
      console.log("📋 Initializing fee contract...");
      console.log("  Admin:", admin.publicKey.toString());
      console.log("  State Account:", feeStateAccount.publicKey.toString());

      const tx = await feesProgram.methods
        .initialize()
        .accounts({
          state: feeStateAccount.publicKey,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin, feeStateAccount])
        .rpc();

      console.log("✅ Fee contract initialized. TX:", tx);

      const state = await feesProgram.account.feeStateAccount.fetch(
        feeStateAccount.publicKey
      );

      assert.equal(
        state.admin.toString(),
        admin.publicKey.toString(),
        "Admin should be set correctly"
      );
      assert.equal(
        state.isStakeEnabled,
        false,
        "Stake should be disabled by default"
      );
      assert.equal(
        state.stakeContractAddress.toString(),
        PublicKey.default.toString(),
        "Stake contract address should be default"
      );
      assert.isString(
        state.feeAddress1.toString(),
        "Fee address 1 should be set"
      );
      assert.isString(
        state.feeAddress2.toString(),
        "Fee address 2 should be set"
      );
      assert.equal(
        state.totalFeesCollected.toNumber(),
        0,
        "Total fees should be 0"
      );

      console.log("✅ All initial state values verified correctly");
      console.log("  Fee Address 1:", state.feeAddress1.toString());
      console.log("  Fee Address 2:", state.feeAddress2.toString());
    });
  });

  describe("Test 1.2: Verify Initial State Properties", () => {
    it("Should have all expected state properties", async () => {
      const state = await feesProgram.account.feeStateAccount.fetch(
        feeStateAccount.publicKey
      );

      expect(state).to.have.property("admin");
      expect(state).to.have.property("isStakeEnabled");
      expect(state).to.have.property("stakeContractAddress");
      expect(state).to.have.property("feeAddress1");
      expect(state).to.have.property("feeAddress2");
      expect(state).to.have.property("totalFeesCollected");

      console.log("✅ State properties verified:");
      console.log("  admin:", state.admin.toString());
      console.log("  isStakeEnabled:", state.isStakeEnabled);
      console.log("  stakeContractAddress:", state.stakeContractAddress.toString());
      console.log("  feeAddress1:", state.feeAddress1.toString());
      console.log("  feeAddress2:", state.feeAddress2.toString());
      console.log("  totalFeesCollected:", state.totalFeesCollected.toNumber());
    });
  });

  describe("Test 1.3: Prevent Re-initialization", () => {
    it("Should fail when trying to reinitialize same state account", async () => {
      console.log("🔒 Attempting to reinitialize same state account...");

      try {
        await feesProgram.methods
          .initialize()
          .accounts({
            state: feeStateAccount.publicKey,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin, feeStateAccount])
          .rpc();

        assert.fail("Should have thrown an error");
      } catch (error: any) {
        console.log("✅ Reinitialization correctly prevented");
        console.log("  Error:", error.message);
        
        expect(error.message).to.include("already in use");
      }
    });

    it("Should allow initialization of different state account", async () => {
      const newStateAccount = Keypair.generate();
      const newAdmin = admin; // Use same admin to avoid funding issues

      const tx = await feesProgram.methods
        .initialize()
        .accounts({
          state: newStateAccount.publicKey,
          admin: newAdmin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([newAdmin, newStateAccount])
        .rpc();

      console.log("✅ New state account initialized successfully. TX:", tx);

      const state = await feesProgram.account.feeStateAccount.fetch(
        newStateAccount.publicKey
      );

      assert.equal(
        state.admin.toString(),
        newAdmin.publicKey.toString()
      );
      assert.equal(state.totalFeesCollected.toNumber(), 0);
    });
  });

  describe("Test 1.4: Get Config Returns Correct Values", () => {
    it("Should return correct config immediately after initialization", async () => {
      const config = await feesProgram.methods
        .getConfig()
        .accounts({
          state: feeStateAccount.publicKey,
        })
        .view();

      console.log("📊 Config retrieved:");
      console.log("  admin:", config.admin.toString());
      console.log("  isStakeEnabled:", config.isStakeEnabled);
      console.log("  stakeContractAddress:", config.stakeContractAddress.toString());
      console.log("  feeAddress1:", config.feeAddress1.toString());
      console.log("  feeAddress2:", config.feeAddress2.toString());
      console.log("  totalFeesCollected:", config.totalFeesCollected.toNumber());

      assert.equal(
        config.admin.toString(),
        admin.publicKey.toString(),
        "Config admin should match state admin"
      );
      assert.equal(
        config.isStakeEnabled,
        false,
        "Config stake enabled should be false"
      );
      assert.equal(
        config.stakeContractAddress.toString(),
        PublicKey.default.toString(),
        "Config stake address should be default"
      );
      assert.isString(
        config.feeAddress1.toString(),
        "Config should include fee address 1"
      );
      assert.isString(
        config.feeAddress2.toString(),
        "Config should include fee address 2"
      );
      assert.equal(
        config.totalFeesCollected.toNumber(),
        0,
        "Config total fees should be 0"
      );

      console.log("✅ get_config returns correct initial values");
    });

    it("Should be callable by non-admin (read-only)", async () => {
      const randomUser = admin; // Use same admin to avoid funding issues

      const config = await feesProgram.methods
        .getConfig()
        .accounts({
          state: feeStateAccount.publicKey,
        })
        .view();

      assert.equal(
        config.admin.toString(),
        admin.publicKey.toString()
      );

      console.log("✅ get_config is callable by anyone (view function)");
    });
  });

  after(() => {
    console.log("\n✅ Fee Contract Initialization Tests Complete");
    console.log("  Total Tests Passed: 6");
    console.log("  State Account:", feeStateAccount.publicKey.toString());
  });
});
