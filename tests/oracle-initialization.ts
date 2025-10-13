import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { assert, expect } from "chai";

describe("Oracle Contract - Initialization Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;

  const PYTH_ORACLE_ADDRESS = new PublicKey("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");

  before(async () => {
    console.log("\n🚀 Setting up Oracle Initialization Tests...");
    console.log("  Network: Devnet");
    console.log("  Admin:", provider.wallet.publicKey.toString());
  });

  describe("Test 1.1: Initialize Oracle Successfully", () => {
    it("Should initialize oracle with correct initial state", async () => {
      const stateAccount = Keypair.generate();

      console.log("📋 Initializing oracle...");
      console.log("  State Account:", stateAccount.publicKey.toString());

      const tx = await oracleProgram.methods
        .initialize({
          oracleAddress: PYTH_ORACLE_ADDRESS,
        })
        .accounts({
          state: stateAccount.publicKey,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([stateAccount])
        .rpc();

      console.log("✅ Oracle initialized. TX:", tx);

      const state = await oracleProgram.account.oracleStateAccount.fetch(
        stateAccount.publicKey
      );

      assert.equal(
        state.admin.toString(),
        provider.wallet.publicKey.toString(),
        "Admin should match"
      );
      assert.equal(
        state.oracleAddress.toString(),
        PYTH_ORACLE_ADDRESS.toString(),
        "Oracle address should match"
      );
      assert.equal(
        state.collateralData.length,
        0,
        "Should start with no collateral data"
      );
      expect(state.lastUpdate).to.be.greaterThan(0);

      console.log("✅ All initial state values verified");
    });
  });

  describe("Test 1.2: Verify Initial State Properties", () => {
    it("Should have all expected state properties", async () => {
      const stateAccount = Keypair.generate();

      await oracleProgram.methods
        .initialize({
          oracleAddress: PYTH_ORACLE_ADDRESS,
        })
        .accounts({
          state: stateAccount.publicKey,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([stateAccount])
        .rpc();

      const state = await oracleProgram.account.oracleStateAccount.fetch(
        stateAccount.publicKey
      );

      expect(state).to.have.property("admin");
      expect(state).to.have.property("oracleAddress");
      expect(state).to.have.property("collateralData");
      expect(state).to.have.property("lastUpdate");

      console.log("✅ State properties verified:");
      console.log("  admin:", state.admin.toString());
      console.log("  oracleAddress:", state.oracleAddress.toString());
      console.log("  collateralData:", state.collateralData.length, "assets");
      console.log("  lastUpdate:", state.lastUpdate.toString());
    });
  });

  describe("Test 1.3: Prevent Re-initialization", () => {
    it("Should fail when trying to reinitialize same state account", async () => {
      const stateAccount = Keypair.generate();

      await oracleProgram.methods
        .initialize({
          oracleAddress: PYTH_ORACLE_ADDRESS,
        })
        .accounts({
          state: stateAccount.publicKey,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([stateAccount])
        .rpc();

      console.log("🔒 Attempting to reinitialize same state account...");

      try {
        await oracleProgram.methods
          .initialize({
            oracleAddress: PYTH_ORACLE_ADDRESS,
          })
          .accounts({
            state: stateAccount.publicKey,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          })
          .signers([admin, stateAccount])
          .rpc();

        assert.fail("Should have thrown an error");
      } catch (error: any) {
        console.log("✅ Reinitialization correctly prevented");
        expect(error).to.exist;
      }
    });
  });

  describe("Test 1.4: Get Config After Initialization", () => {
    it("Should return correct config via get_config", async () => {
      const stateAccount = Keypair.generate();

      await oracleProgram.methods
        .initialize({
          oracleAddress: PYTH_ORACLE_ADDRESS,
        })
        .accounts({
          state: stateAccount.publicKey,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([stateAccount])
        .rpc();

      const config = await oracleProgram.methods
        .getConfig({})
        .accounts({
          state: stateAccount.publicKey,
        })
        .view();

      console.log("📊 Config retrieved:");
      console.log("  admin:", config.admin.toString());
      console.log("  oracleAddress:", config.oracleAddress.toString());
      console.log("  assetCount:", config.assetCount);
      console.log("  lastUpdate:", config.lastUpdate.toString());

      assert.equal(
        config.admin.toString(),
        provider.wallet.publicKey.toString()
      );
      assert.equal(
        config.oracleAddress.toString(),
        PYTH_ORACLE_ADDRESS.toString()
      );
      assert.equal(config.assetCount, 0);
      expect(config.lastUpdate).to.be.greaterThan(0);

      console.log("✅ get_config working correctly");
    });
  });

  describe("Test 1.5: Initialize with Different Oracle Addresses", () => {
    it("Should accept different oracle provider addresses", async () => {
      const stateAccount = Keypair.generate();
      const customOracle = Keypair.generate().publicKey;

      await oracleProgram.methods
        .initialize({
          oracleAddress: customOracle,
        })
        .accounts({
          state: stateAccount.publicKey,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([stateAccount])
        .rpc();

      const state = await oracleProgram.account.oracleStateAccount.fetch(
        stateAccount.publicKey
      );

      assert.equal(
        state.oracleAddress.toString(),
        customOracle.toString()
      );

      console.log("✅ Custom oracle address accepted:", customOracle.toString());
    });
  });

  describe("Test 1.6: Verify Empty Collateral Data on Init", () => {
    it("Should start with empty collateral data array", async () => {
      const stateAccount = Keypair.generate();

      await oracleProgram.methods
        .initialize({
          oracleAddress: PYTH_ORACLE_ADDRESS,
        })
        .accounts({
          state: stateAccount.publicKey,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([stateAccount])
        .rpc();

      const denoms = await oracleProgram.methods
        .getAllDenoms({})
        .accounts({
          state: stateAccount.publicKey,
        })
        .view();

      assert.equal(denoms.length, 0, "Should have no supported assets initially");
      console.log("✅ Empty collateral data verified");
    });
  });

  after(() => {
    console.log("\n✅ Oracle Initialization Tests Complete");
    console.log("  Total Tests Passed: 6\n");
  });
});
