import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram, 
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import { assert } from "chai";

describe("Fee Contract Verification Test", () => {
  // Configure the client to use local cluster
  anchor.setProvider(anchor.AnchorProvider.local());

  const provider = anchor.getProvider();
  const connection = provider.connection;

  // Test accounts
  const admin = (provider as any).wallet.payer;

  before(async () => {
    console.log("Setting up fee contract verification test...");
    
    // Check admin balance
    const adminBalance = await connection.getBalance(admin.publicKey);
    console.log("Admin balance:", adminBalance / LAMPORTS_PER_SOL, "SOL");
  });

  describe("Program Accessibility", () => {
    it("Should be able to access the fee program", async () => {
      try {
        const feesProgram = anchor.workspace.AerospacerFees as Program<any>;
        console.log("✅ Fee program is accessible");
        
        // Check if we can get program info
        const programId = feesProgram.programId;
        console.log("Program ID:", programId.toString());
        
        // Verify program exists on chain
        const programInfo = await connection.getAccountInfo(programId);
        if (programInfo) {
          console.log("✅ Program exists on chain");
          console.log("Program data length:", programInfo.data.length);
        } else {
          console.log("❌ Program not found on chain");
        }
        
      } catch (error) {
        console.error("❌ Cannot access fee program:", error);
        throw error;
      }
    });

    it("Should be able to create a state account", async () => {
      try {
        const feesProgram = anchor.workspace.AerospacerFees as Program<any>;
        const feeStateAccount = Keypair.generate();
        
        console.log("Creating state account:", feeStateAccount.publicKey.toString());
        
        await feesProgram.methods.initialize()
          .accounts({
            state: feeStateAccount.publicKey,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([feeStateAccount])
          .rpc();

        console.log("✅ State account created successfully");
        
        // Try to fetch the account
        const state = await feesProgram.account.feeStateAccount.fetch(feeStateAccount.publicKey);
        console.log("✅ State account fetched successfully");
        console.log("Admin:", state.admin.toString());
        console.log("Stake enabled:", state.isStakeEnabled);
        console.log("Total fees:", state.totalFeesCollected.toString());
        
      } catch (error) {
        console.error("❌ Failed to create state account:", error);
        throw error;
      }
    });
  });

  after(async () => {
    console.log("\n🎉 Fee contract verification completed!");
  });
});
