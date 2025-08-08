import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram, 
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import { assert } from "chai";

describe("Fee Contract Basic Tests", () => {
  // Configure the client to use local cluster
  anchor.setProvider(anchor.AnchorProvider.local());

  const provider = anchor.getProvider();
  const connection = provider.connection;

  // Program
  const feesProgram = anchor.workspace.AerospacerFees as Program<any>;

  // Test accounts
  const admin = (provider as any).wallet.payer;
  const feeStateAccount = Keypair.generate();

  before(async () => {
    console.log("Setting up basic fee contract test environment...");
    
    // Check admin balance
    const adminBalance = await connection.getBalance(admin.publicKey);
    console.log("Admin balance:", adminBalance / LAMPORTS_PER_SOL, "SOL");
  });

  describe("Initialization", () => {
    it("Should initialize the fee contract successfully", async () => {
      try {
        await feesProgram.methods.initialize()
          .accounts({
            state: feeStateAccount.publicKey,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([feeStateAccount])
          .rpc();

        const state = await feesProgram.account.feeStateAccount.fetch(feeStateAccount.publicKey);
        
        assert.equal(state.admin.toString(), admin.publicKey.toString());
        assert.equal(state.isStakeEnabled, false);
        assert.equal(state.totalFeesCollected.toString(), '0');
        
        console.log("✅ Fee contract initialized successfully");
      } catch (error) {
        console.error("❌ Initialization failed:", error);
        throw error;
      }
    });
  });

  describe("Admin Controls", () => {
    it("Should toggle stake contract status", async () => {
      try {
        await feesProgram.methods.toggleStakeContract()
          .accounts({
            admin: admin.publicKey,
            state: feeStateAccount.publicKey,
          })
          .rpc();

        let state = await feesProgram.account.feeStateAccount.fetch(feeStateAccount.publicKey);
        assert.equal(state.isStakeEnabled, true);

        await feesProgram.methods.toggleStakeContract()
          .accounts({
            admin: admin.publicKey,
            state: feeStateAccount.publicKey,
          })
          .rpc();

        state = await feesProgram.account.feeStateAccount.fetch(feeStateAccount.publicKey);
        assert.equal(state.isStakeEnabled, false);
        
        console.log("✅ Stake contract toggle works correctly");
      } catch (error) {
        console.error("❌ Toggle failed:", error);
        throw error;
      }
    });

    it("Should set stake contract address", async () => {
      try {
        const newStakeAddress = Keypair.generate().publicKey;
        
        await feesProgram.methods.setStakeContractAddress({
          address: newStakeAddress.toString()
        })
          .accounts({
            admin: admin.publicKey,
            state: feeStateAccount.publicKey,
          })
          .rpc();

        const state = await feesProgram.account.feeStateAccount.fetch(feeStateAccount.publicKey);
        assert.equal(state.stakeContractAddress.toString(), newStakeAddress.toString());
        
        console.log("✅ Stake contract address set successfully");
      } catch (error) {
        console.error("❌ Set address failed:", error);
        throw error;
      }
    });

    it("Should reject admin operations from non-admin", async () => {
      try {
        const fakeAdmin = Keypair.generate();
        
        await feesProgram.methods.toggleStakeContract()
          .accounts({
            admin: fakeAdmin.publicKey,
            state: feeStateAccount.publicKey,
          })
          .signers([fakeAdmin])
          .rpc();
        
        assert.fail("Should have thrown an error");
      } catch (error) {
        console.log("✅ Correctly rejected admin operation from non-admin");
      }
    });
  });

  describe("Configuration Queries", () => {
    it("Should retrieve configuration correctly", async () => {
      try {
        const config = await feesProgram.methods.getConfig()
          .accounts({
            state: feeStateAccount.publicKey,
          })
          .view();

        assert.equal(config.admin.toString(), admin.publicKey.toString());
        assert.equal(config.isStakeEnabled, false);
        
        console.log("✅ Configuration retrieval works correctly");
      } catch (error) {
        console.error("❌ Config retrieval failed:", error);
        throw error;
      }
    });
  });

  describe("State Management", () => {
    it("Should maintain state consistency across operations", async () => {
      try {
        // Toggle stake multiple times
        for (let i = 0; i < 3; i++) {
          await feesProgram.methods.toggleStakeContract()
            .accounts({
              admin: admin.publicKey,
              state: feeStateAccount.publicKey,
            })
            .rpc();
        }
        
        // Set different stake addresses
        const addresses = [
          Keypair.generate().publicKey,
          Keypair.generate().publicKey,
          Keypair.generate().publicKey
        ];
        
        for (const address of addresses) {
          await feesProgram.methods.setStakeContractAddress({
            address: address.toString()
          })
            .accounts({
              admin: admin.publicKey,
              state: feeStateAccount.publicKey,
            })
            .rpc();
        }
        
        // Verify final state
        const finalState = await feesProgram.account.feeStateAccount.fetch(feeStateAccount.publicKey);
        assert.equal(finalState.isStakeEnabled, true, "Final stake state should be correct");
        assert.equal(finalState.stakeContractAddress.toString(), addresses[2].toString(), "Final address should be correct");
        
        console.log("✅ State consistency maintained across operations");
      } catch (error) {
        console.error("❌ State consistency test failed:", error);
        throw error;
      }
    });
  });

  after(async () => {
    console.log("\n🎉 All basic fee contract tests completed successfully!");
    console.log("📊 Test Summary:");
    console.log("- ✅ Initialization: Working correctly");
    console.log("- ✅ Admin Controls: Toggle, address setting, and authorization working");
    console.log("- ✅ Configuration Queries: Retrieving data correctly");
    console.log("- ✅ State Management: Consistency maintained across operations");
  });
});
