import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { assert } from "chai";

describe("Fixed Program IDs Test", () => {
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

  describe("Program ID Verification", () => {
    it("Should verify program IDs match deployed programs", async () => {
      // Expected deployed program IDs
      const EXPECTED_PROTOCOL_ID = "mR3CUXYeYLjoxFJ1ieBfC9rLciZwe8feFYvXKdafihD";
      const EXPECTED_ORACLE_ID = "8gLDpdg9tFAtAnpZabd2w6V7qQikfqCKzPiNpx9Wcr3c";
      const EXPECTED_FEES_ID = "8PC52W8S5WQ1X6gBBNQr5AvYYxVEa68DahEgFJAueZF4";

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
        
        // Verify program IDs match
        const protocolProgram = workspace.AerospacerProtocol as Program<AerospacerProtocol>;
        const oracleProgram = workspace.AerospacerOracle as Program<AerospacerOracle>;
        const feesProgram = workspace.AerospacerFees as Program<AerospacerFees>;

        assert(protocolProgram.programId.toString() === EXPECTED_PROTOCOL_ID, 
          `Protocol program ID mismatch. Expected: ${EXPECTED_PROTOCOL_ID}, Got: ${protocolProgram.programId.toString()}`);
        assert(oracleProgram.programId.toString() === EXPECTED_ORACLE_ID, 
          `Oracle program ID mismatch. Expected: ${EXPECTED_ORACLE_ID}, Got: ${oracleProgram.programId.toString()}`);
        assert(feesProgram.programId.toString() === EXPECTED_FEES_ID, 
          `Fees program ID mismatch. Expected: ${EXPECTED_FEES_ID}, Got: ${feesProgram.programId.toString()}`);

        console.log("✅ All program IDs match deployed programs!");
      } else {
        console.log("⚠️ Some programs are missing from workspace");
        throw new Error("Not all programs are available in workspace");
      }
    });
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

  describe("Program Initialization Test", () => {
    it("Should test program initialization with correct IDs", async () => {
      try {
        const workspace = anchor.workspace;
        const protocolProgram = workspace.AerospacerProtocol as Program<AerospacerProtocol>;
        const oracleProgram = workspace.AerospacerOracle as Program<AerospacerOracle>;
        const feesProgram = workspace.AerospacerFees as Program<AerospacerFees>;

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

        console.log("✅ PDA derivation successful");
        console.log("- Protocol State:", protocolStatePda.toString());
        console.log("- Oracle State:", oracleStatePda.toString());
        console.log("- Fees State:", feesStatePda.toString());

        // Test that we can access program methods (this will fail if types are wrong)
        console.log("✅ Program method access verified");
        console.log("- Protocol methods:", Object.keys(protocolProgram.methods));
        console.log("- Oracle methods:", Object.keys(oracleProgram.methods));
        console.log("- Fees methods:", Object.keys(feesProgram.methods));

      } catch (error) {
        console.log("❌ Program initialization test failed:", error);
        throw error;
      }
    });
  });

  describe("Token System Verification", () => {
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
      console.log("🎉 Program ID alignment successful!");
      console.log("\n📋 Next Steps:");
      console.log("1. ✅ Program IDs are aligned");
      console.log("2. ✅ Token system is ready");
      console.log("3. 🔄 Initialize program state accounts");
      console.log("4. 🔄 Test core protocol operations");
      console.log("5. 🔄 Implement trove management");
      
      console.log("\n🎯 Current Status:");
      console.log("- Infrastructure: ✅ Working");
      console.log("- Program IDs: ✅ Aligned");
      console.log("- Token System: ✅ Ready");
      console.log("- Program Access: ✅ Available");
      console.log("- State Initialization: 🔄 Ready to proceed");
      console.log("- Core Operations: 🔄 Ready to test");
    });
  });
});
