import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { assert } from "chai";

describe("Program Initialization", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Use the actual deployed program IDs
  const PROTOCOL_PROGRAM_ID = "mR3CUXYeYLjoxFJ1ieBfC9rLciZwe8feFYvXKdafihD";
  const ORACLE_PROGRAM_ID = "8gLDpdg9tFAtAnpZabd2w6V7qQikfqCKzPiNpx9Wcr3c";
  const FEES_PROGRAM_ID = "8PC52W8S5WQ1X6gBBNQr5AvYYxVEa68DahEgFJAueZF4";

  // Test accounts
  const admin = Keypair.generate();
  const user1 = Keypair.generate();

  // Program state accounts
  let protocolState: PublicKey;
  let oracleState: PublicKey;
  let feesState: PublicKey;

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

    // Derive state PDAs
    const [protocolStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      new PublicKey(PROTOCOL_PROGRAM_ID)
    );
    const [oracleStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      new PublicKey(ORACLE_PROGRAM_ID)
    );
    const [feesStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      new PublicKey(FEES_PROGRAM_ID)
    );

    protocolState = protocolStatePda;
    oracleState = oracleStatePda;
    feesState = feesStatePda;

    console.log("✅ Setup completed");
    console.log("- Admin:", admin.publicKey.toString());
    console.log("- User1:", user1.publicKey.toString());
    console.log("- Protocol State:", protocolState.toString());
    console.log("- Oracle State:", oracleState.toString());
    console.log("- Fees State:", feesState.toString());
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

  describe("Oracle Program Initialization", () => {
    it("Should initialize oracle program", async () => {
      try {
        // Create a dummy oracle address for testing
        const dummyOracleAddress = Keypair.generate().publicKey;

        // We need to create the instruction manually since we don't have the program types
        const initializeIx = {
          programId: new PublicKey(ORACLE_PROGRAM_ID),
          keys: [
            { pubkey: oracleState, isSigner: true, isWritable: true },
            { pubkey: admin.publicKey, isSigner: true, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
          ],
          data: Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]) // Initialize discriminator
        };

        const transaction = new anchor.web3.Transaction().add(initializeIx);
        const signature = await anchor.web3.sendAndConfirmTransaction(
          provider.connection,
          transaction,
          [admin]
        );

        console.log("✅ Oracle program initialized");
        console.log("- Transaction signature:", signature);
      } catch (error) {
        console.log("❌ Oracle initialization failed:", error);
        throw error;
      }
    });
  });

  describe("Fees Program Initialization", () => {
    it("Should initialize fees program", async () => {
      try {
        // Create the instruction manually
        const initializeIx = {
          programId: new PublicKey(FEES_PROGRAM_ID),
          keys: [
            { pubkey: feesState, isSigner: true, isWritable: true },
            { pubkey: admin.publicKey, isSigner: true, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
          ],
          data: Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]) // Initialize discriminator
        };

        const transaction = new anchor.web3.Transaction().add(initializeIx);
        const signature = await anchor.web3.sendAndConfirmTransaction(
          provider.connection,
          transaction,
          [admin]
        );

        console.log("✅ Fees program initialized");
        console.log("- Transaction signature:", signature);
      } catch (error) {
        console.log("❌ Fees initialization failed:", error);
        throw error;
      }
    });
  });

  describe("Protocol Program Initialization", () => {
    it("Should initialize protocol program", async () => {
      try {
        // Create the instruction manually with proper parameters
        const initializeParams = {
          stableCoinMint: stablecoinMint,
          oracleProgram: new PublicKey(ORACLE_PROGRAM_ID),
          feeDistributor: new PublicKey(FEES_PROGRAM_ID)
        };

        // Serialize the parameters (this is a simplified version)
        const paramsBuffer = Buffer.alloc(96); // 32 + 32 + 32 bytes for 3 pubkeys
        paramsBuffer.set(initializeParams.stableCoinMint.toBytes(), 0);
        paramsBuffer.set(initializeParams.oracleProgram.toBytes(), 32);
        paramsBuffer.set(initializeParams.feeDistributor.toBytes(), 64);

        const initializeIx = {
          programId: new PublicKey(PROTOCOL_PROGRAM_ID),
          keys: [
            { pubkey: protocolState, isSigner: true, isWritable: true },
            { pubkey: admin.publicKey, isSigner: true, isWritable: true },
            { pubkey: stablecoinMint, isSigner: false, isWritable: false },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
          ],
          data: Buffer.concat([
            Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]), // Initialize discriminator
            paramsBuffer
          ])
        };

        const transaction = new anchor.web3.Transaction().add(initializeIx);
        const signature = await anchor.web3.sendAndConfirmTransaction(
          provider.connection,
          transaction,
          [admin]
        );

        console.log("✅ Protocol program initialized");
        console.log("- Transaction signature:", signature);
      } catch (error) {
        console.log("❌ Protocol initialization failed:", error);
        throw error;
      }
    });
  });

  describe("State Verification", () => {
    it("Should verify all state accounts exist", async () => {
      try {
        const protocolAccount = await provider.connection.getAccountInfo(protocolState);
        const oracleAccount = await provider.connection.getAccountInfo(oracleState);
        const feesAccount = await provider.connection.getAccountInfo(feesState);

        assert(protocolAccount !== null, "Protocol state account should exist");
        assert(oracleAccount !== null, "Oracle state account should exist");
        assert(feesAccount !== null, "Fees state account should exist");

        console.log("✅ All state accounts verified");
        console.log("- Protocol state size:", protocolAccount?.data.length, "bytes");
        console.log("- Oracle state size:", oracleAccount?.data.length, "bytes");
        console.log("- Fees state size:", feesAccount?.data.length, "bytes");
      } catch (error) {
        console.log("❌ State verification failed:", error);
        throw error;
      }
    });

    it("Should verify stablecoin system", async () => {
      try {
        const mintAccount = await provider.connection.getAccountInfo(stablecoinMint);
        const adminAccount = await provider.connection.getAccountInfo(adminStablecoinAccount);
        const user1Account = await provider.connection.getAccountInfo(user1StablecoinAccount);

        assert(mintAccount !== null, "Stablecoin mint should exist");
        assert(adminAccount !== null, "Admin stablecoin account should exist");
        assert(user1Account !== null, "User1 stablecoin account should exist");

        console.log("✅ Stablecoin system verified");
        console.log("- Mint account size:", mintAccount?.data.length, "bytes");
        console.log("- Admin account size:", adminAccount?.data.length, "bytes");
        console.log("- User1 account size:", user1Account?.data.length, "bytes");
      } catch (error) {
        console.log("❌ Stablecoin verification failed:", error);
        throw error;
      }
    });
  });
});
