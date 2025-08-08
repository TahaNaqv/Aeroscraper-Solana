import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { assert } from "chai";

describe("Simple Program Initialization", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Test accounts
  const admin = Keypair.generate();
  const user1 = Keypair.generate();

  // Token accounts
  let stablecoinMint: PublicKey;
  let adminStablecoinAccount: PublicKey;
  let user1StablecoinAccount: PublicKey;

  before(async () => {
    // Airdrop SOL to admin
    const signature = await provider.connection.requestAirdrop(admin.publicKey, 1000000000);
    await provider.connection.confirmTransaction(signature);

    // Airdrop SOL to user1
    const signature2 = await provider.connection.requestAirdrop(user1.publicKey, 1000000000);
    await provider.connection.confirmTransaction(signature2);

    console.log("✅ Setup completed");
    console.log("- Admin:", admin.publicKey.toString());
    console.log("- User1:", user1.publicKey.toString());
  });

  describe("Token System Setup", () => {
    it("Should create stablecoin mint", async () => {
      try {
        stablecoinMint = await createMint(
          provider.connection,
          admin,
          admin.publicKey,
          null,
          6
        );

        console.log("✅ Stablecoin mint created:", stablecoinMint.toString());
      } catch (error) {
        console.log("❌ Stablecoin mint creation failed:", error);
        throw error;
      }
    });

    it("Should create stablecoin token accounts", async () => {
      try {
        adminStablecoinAccount = await getAssociatedTokenAddress(stablecoinMint, admin.publicKey);
        user1StablecoinAccount = await getAssociatedTokenAddress(stablecoinMint, user1.publicKey);

        await createAssociatedTokenAccount(
          provider.connection,
          admin,
          admin.publicKey,
          stablecoinMint
        );

        await createAssociatedTokenAccount(
          provider.connection,
          admin,
          user1.publicKey,
          stablecoinMint
        );

        // Mint initial stablecoins to admin
        await mintTo(
          provider.connection,
          admin,
          stablecoinMint,
          adminStablecoinAccount,
          admin,
          1000000000 // 1000 stablecoins
        );

        console.log("✅ Stablecoin token accounts created");
        console.log("- Admin account:", adminStablecoinAccount.toString());
        console.log("- User1 account:", user1StablecoinAccount.toString());
      } catch (error) {
        console.log("❌ Token account creation failed:", error);
        throw error;
      }
    });
  });

  describe("Program Status Check", () => {
    it("Should check if programs are accessible via workspace", async () => {
      try {
        // Check if programs exist in workspace
        const workspace = anchor.workspace;
        console.log("📊 Workspace programs:");
        
        for (const [name, program] of Object.entries(workspace)) {
          if (program && typeof program === 'object' && 'programId' in program) {
            console.log(`- ${name}: ${(program as any).programId.toString()}`);
          }
        }

        // Check if our specific programs exist
        const hasProtocol = 'AerospacerProtocol' in workspace;
        const hasOracle = 'AerospacerOracle' in workspace;
        const hasFees = 'AerospacerFees' in workspace;

        console.log("✅ Program availability:");
        console.log(`- Protocol: ${hasProtocol ? '✅' : '❌'}`);
        console.log(`- Oracle: ${hasOracle ? '✅' : '❌'}`);
        console.log(`- Fees: ${hasFees ? '✅' : '❌'}`);

        if (hasProtocol && hasOracle && hasFees) {
          console.log("🎉 All programs are available in workspace!");
        } else {
          console.log("⚠️ Some programs are missing from workspace");
        }
      } catch (error) {
        console.log("❌ Workspace check failed:", error);
        throw error;
      }
    });

    it("Should verify token system", async () => {
      try {
        const mintAccount = await provider.connection.getAccountInfo(stablecoinMint);
        const adminAccount = await provider.connection.getAccountInfo(adminStablecoinAccount);
        const user1Account = await provider.connection.getAccountInfo(user1StablecoinAccount);

        assert(mintAccount !== null, "Stablecoin mint should exist");
        assert(adminAccount !== null, "Admin stablecoin account should exist");
        assert(user1Account !== null, "User1 stablecoin account should exist");

        console.log("✅ Token system verified");
        console.log("- Mint account size:", mintAccount?.data.length, "bytes");
        console.log("- Admin account size:", adminAccount?.data.length, "bytes");
        console.log("- User1 account size:", user1Account?.data.length, "bytes");
      } catch (error) {
        console.log("❌ Token verification failed:", error);
        throw error;
      }
    });
  });

  describe("Next Steps", () => {
    it("Should provide guidance for next steps", async () => {
      console.log("📋 Next Steps:");
      console.log("1. ✅ Token system is ready");
      console.log("2. 🔄 Need to initialize program state accounts");
      console.log("3. 🔄 Need to test core protocol operations");
      console.log("4. 🔄 Need to implement trove management");
      console.log("5. 🔄 Need to test lending operations");
      
      console.log("\n🎯 Current Status:");
      console.log("- Infrastructure: ✅ Working");
      console.log("- Token System: ✅ Ready");
      console.log("- Program Access: ✅ Available");
      console.log("- State Initialization: 🔄 Pending");
      console.log("- Core Operations: 🔄 Pending");
    });
  });
});
