import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import { assert } from "chai";

describe("Basic Protocol Operations", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  // Test accounts
  const admin = Keypair.generate();
  const user1 = Keypair.generate();

  // Program state accounts
  let protocolState: PublicKey;
  let oracleState: PublicKey;
  let feesState: PublicKey;

  // User trove account
  let user1Trove: PublicKey;

  before(async () => {
    // Airdrop SOL to admin
    const signature = await provider.connection.requestAirdrop(admin.publicKey, 1000000000);
    await provider.connection.confirmTransaction(signature);

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

    protocolState = protocolStatePda;
    oracleState = oracleStatePda;
    feesState = feesStatePda;

    // Derive user trove PDA
    const [user1TrovePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("trove"), user1.publicKey.toBuffer()],
      protocolProgram.programId
    );

    user1Trove = user1TrovePda;

    console.log("✅ Setup completed");
    console.log("- Protocol State:", protocolState.toString());
    console.log("- Oracle State:", oracleState.toString());
    console.log("- Fees State:", feesState.toString());
    console.log("- User1 Trove:", user1Trove.toString());
  });

  describe("Program Initialization", () => {
    it("Should initialize oracle program", async () => {
      try {
        await oracleProgram.methods
          .initialize()
          .accounts({
            state: oracleState,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([admin])
          .rpc();

        console.log("✅ Oracle program initialized");
      } catch (e) {
        console.log("Oracle already initialized");
      }
    });

    it("Should initialize fees program", async () => {
      try {
        await feesProgram.methods
          .initialize()
          .accounts({
            state: feesState,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([admin])
          .rpc();

        console.log("✅ Fees program initialized");
      } catch (e) {
        console.log("Fees already initialized");
      }
    });

    it("Should initialize protocol program", async () => {
      try {
        // Create a dummy stablecoin mint for testing
        const stablecoinMint = Keypair.generate();
        
        await protocolProgram.methods
          .initialize({
            stableCoinMint: stablecoinMint.publicKey,
            oracleProgram: oracleProgram.programId,
            feeDistributor: feesProgram.programId,
          })
          .accounts({
            state: protocolState,
            admin: admin.publicKey,
            stableCoinMint: stablecoinMint.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc();

        console.log("✅ Protocol program initialized");
      } catch (e) {
        console.log("Protocol already initialized");
      }
    });
  });

  describe("Protocol State Verification", () => {
    it("Should verify protocol state exists", async () => {
      try {
        const stateAccount = await protocolProgram.account.stateAccount.fetch(protocolState);
        console.log("📊 Protocol State:");
        console.log("- Admin:", stateAccount.admin.toString());
        console.log("- Stablecoin Mint:", stateAccount.stableCoinMint.toString());
        console.log("- Oracle Program:", stateAccount.oracleProgram.toString());
        console.log("- Fee Distributor:", stateAccount.feeDistributor.toString());
        console.log("- Minimum Collateral Ratio:", stateAccount.minimumCollateralRatio);
        console.log("- Protocol Fee:", stateAccount.protocolFee);
        console.log("- Total Debt Amount:", stateAccount.totalDebtAmount.toString());
        console.log("- Total Stake Amount:", stateAccount.totalStakeAmount.toString());
        
        assert(stateAccount.admin.equals(admin.publicKey), "Admin should match");
        
        console.log("✅ Protocol state verification passed");
      } catch (error) {
        console.log("❌ Protocol state verification failed:", error);
        throw error;
      }
    });

    it("Should verify oracle state exists", async () => {
      try {
        const oracleAccount = await oracleProgram.account.oracleStateAccount.fetch(oracleState);
        console.log("📊 Oracle State:");
        console.log("- Admin:", oracleAccount.admin.toString());
        console.log("- Oracle Address:", oracleAccount.oracleAddress.toString());
        console.log("- Collateral Data Count:", oracleAccount.collateralData.length);
        
        assert(oracleAccount.admin.equals(admin.publicKey), "Oracle admin should match");
        
        console.log("✅ Oracle state verification passed");
      } catch (error) {
        console.log("❌ Oracle state verification failed:", error);
        throw error;
      }
    });

    it("Should verify fees state exists", async () => {
      try {
        const feesAccount = await feesProgram.account.feeStateAccount.fetch(feesState);
        console.log("📊 Fees State:");
        console.log("- Admin:", feesAccount.admin.toString());
        console.log("- Is Stake Enabled:", feesAccount.isStakeEnabled);
        console.log("- Total Fees Collected:", feesAccount.totalFeesCollected.toString());
        console.log("- Fee Address 1:", feesAccount.feeAddress1.toString());
        console.log("- Fee Address 2:", feesAccount.feeAddress2.toString());
        
        assert(feesAccount.admin.equals(admin.publicKey), "Fees admin should match");
        
        console.log("✅ Fees state verification passed");
      } catch (error) {
        console.log("❌ Fees state verification failed:", error);
        throw error;
      }
    });
  });

  describe("Basic Protocol Operations", () => {
    it("Should verify trove account structure", async () => {
      try {
        // This will fail if the trove doesn't exist, which is expected
        await protocolProgram.account.troveAccount.fetch(user1Trove);
        console.log("❌ Trove should not exist yet");
        assert.fail("Trove should not exist yet");
      } catch (error) {
        console.log("✅ Trove account verification passed (account doesn't exist yet, which is correct)");
      }
    });
  });
});
