import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { PublicKey, Keypair, SystemProgram, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { expect } from "chai";

describe("Aerospacer Oracle Contract", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;

  // Test accounts
  const admin = Keypair.generate();
  const oracleState = Keypair.generate();
  const mockPythPriceFeed = Keypair.generate();

  // Test data - using a valid 64-character hex string without "0x" prefix
  const testOracleAddress = Keypair.generate().publicKey;
  const testDenom = "SOL";
  const testDecimal = 9;
  const testPriceId = "1".repeat(64); // 64 character hex string without 0x prefix

  before(async () => {
    // Airdrop SOL to admin
    const signature = await provider.connection.requestAirdrop(admin.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature);
  });

  it("Should initialize the oracle contract", async () => {
    try {
      await program.methods
        .initialize({
          oracleAddress: testOracleAddress,
        })
        .accounts({
          state: oracleState.publicKey,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([admin, oracleState])
        .rpc();

      // Verify state
      const state = await program.account.oracleStateAccount.fetch(oracleState.publicKey);
      expect(state.admin.toString()).to.equal(admin.publicKey.toString());
      expect(state.oracleAddress.toString()).to.equal(testOracleAddress.toString());
      expect(state.collateralData.length).to.equal(0);

      console.log("✅ Oracle contract initialized successfully");
    } catch (error) {
      console.error("❌ Initialize failed:", error);
      throw error;
    }
  });

  it("Should set data for a single collateral asset", async () => {
    try {
      await program.methods
        .setData({
          denom: testDenom,
          decimal: testDecimal,
          priceId: testPriceId,
          pythPriceAccount: mockPythPriceFeed.publicKey,
        })
        .accounts({
          admin: admin.publicKey,
          state: oracleState.publicKey,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([admin])
        .rpc();

      // Verify data was set
      const state = await program.account.oracleStateAccount.fetch(oracleState.publicKey);
      expect(state.collateralData.length).to.equal(1);
      expect(state.collateralData[0].denom).to.equal(testDenom);
      expect(state.collateralData[0].decimal).to.equal(testDecimal);
      expect(state.collateralData[0].priceId).to.equal(testPriceId);

      console.log("✅ Single asset data set successfully");
    } catch (error) {
      console.error("❌ Set data failed:", error);
      throw error;
    }
  });

  it("Should get configuration", async () => {
    try {
      const config = await program.methods
        .getConfig({})
        .accounts({
          state: oracleState.publicKey,
        })
        .view();

      expect(config.admin.toString()).to.equal(admin.publicKey.toString());
      expect(config.oracleAddress.toString()).to.equal(testOracleAddress.toString());
      expect(config.assetCount).to.equal(1);

      console.log("✅ Configuration retrieved successfully");
    } catch (error) {
      console.error("❌ Get config failed:", error);
      throw error;
    }
  });

  it("Should get all denominations", async () => {
    try {
      const denoms = await program.methods
        .getAllDenoms({})
        .accounts({
          state: oracleState.publicKey,
        })
        .view();

      expect(denoms.length).to.equal(1);
      expect(denoms[0]).to.equal(testDenom);

      console.log("✅ All denominations retrieved successfully");
    } catch (error) {
      console.error("❌ Get all denoms failed:", error);
      throw error;
    }
  });

  it("Should check if denom exists", async () => {
    try {
      const exists = await program.methods
        .checkDenom({
          denom: testDenom,
        })
        .accounts({
          state: oracleState.publicKey,
        })
        .view();

      expect(exists).to.be.true;

      console.log("✅ Denom check successful");
    } catch (error) {
      console.error("❌ Check denom failed:", error);
      throw error;
    }
  });

  it("Should get price ID for denom", async () => {
    try {
      const priceId = await program.methods
        .getPriceId({
          denom: testDenom,
        })
        .accounts({
          state: oracleState.publicKey,
        })
        .view();

      expect(priceId).to.equal(testPriceId);

      console.log("✅ Price ID retrieved successfully");
    } catch (error) {
      console.error("❌ Get price ID failed:", error);
      throw error;
    }
  });

  console.log("🎉 All basic oracle tests completed successfully!");
});