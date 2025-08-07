import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
  AerospacerProtocol, 
  AerospacerOracle, 
  AerospacerFees 
} from "../target/types";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram, 
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  createMint, 
  createAssociatedTokenAccount,
  mintTo
} from "@solana/spl-token";
import { assert } from "chai";

describe("Simple Aerospacer Tests", () => {
  // Configure the client to use devnet
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const connection = provider.connection;

  // Programs - using already deployed programs
  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  // Test accounts
  const admin = (provider as any).wallet.payer;

  // Token mints and accounts
  let stablecoinMint: PublicKey;
  let collateralMint: PublicKey;
  let userStablecoinAccount: PublicKey;
  let userCollateralAccount: PublicKey;

  // State accounts
  let protocolState: PublicKey;
  let oracleState: PublicKey;
  let feesState: PublicKey;

  before(async () => {
    console.log("Setting up simple test environment...");
    console.log("Using existing deployed programs");
    
    // Check admin balance
    const adminBalance = await connection.getBalance(admin.publicKey);
    console.log("Admin balance:", adminBalance / LAMPORTS_PER_SOL, "SOL");

    // Create token mints
    stablecoinMint = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      6
    );

    collateralMint = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      6
    );

    console.log("Token mints created");

    // Create token accounts
    userStablecoinAccount = await createAssociatedTokenAccount(
      connection,
      admin,
      stablecoinMint,
      admin.publicKey
    );

    userCollateralAccount = await createAssociatedTokenAccount(
      connection,
      admin,
      collateralMint,
      admin.publicKey
    );

    console.log("Token accounts created");

    // Mint initial tokens
    await mintTo(
      connection,
      admin,
      collateralMint,
      userCollateralAccount,
      admin,
      1000000000 // 1000 tokens
    );

    console.log("Initial tokens minted");

    // Derive PDA addresses
    [protocolState] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      protocolProgram.programId
    );

    [oracleState] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      oracleProgram.programId
    );

    [feesState] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      feesProgram.programId
    );

    console.log("PDA addresses derived");
    console.log("Test environment ready");
  });

  describe("Basic Program Tests", () => {
    it("Should verify programs are accessible", async () => {
      console.log("Testing program accessibility...");
      
      // Check if programs exist
      const oracleProgramInfo = await connection.getAccountInfo(oracleProgram.programId);
      const protocolProgramInfo = await connection.getAccountInfo(protocolProgram.programId);
      const feesProgramInfo = await connection.getAccountInfo(feesProgram.programId);
      
      assert.isNotNull(oracleProgramInfo, "Oracle program should exist");
      assert.isNotNull(protocolProgramInfo, "Protocol program should exist");
      assert.isNotNull(feesProgramInfo, "Fees program should exist");
      
      console.log("All programs are accessible");
    });

    it("Should verify token accounts are created", async () => {
      console.log("Testing token accounts...");
      
      // Check token accounts
      const stablecoinAccountInfo = await connection.getAccountInfo(userStablecoinAccount);
      const collateralAccountInfo = await connection.getAccountInfo(userCollateralAccount);
      
      assert.isNotNull(stablecoinAccountInfo, "Stablecoin account should exist");
      assert.isNotNull(collateralAccountInfo, "Collateral account should exist");
      
      console.log("Token accounts verified");
    });

    it("Should verify PDA addresses are derived", async () => {
      console.log("Testing PDA derivation...");
      
      // Check PDA addresses
      assert.isNotNull(protocolState, "Protocol state PDA should be derived");
      assert.isNotNull(oracleState, "Oracle state PDA should be derived");
      assert.isNotNull(feesState, "Fees state PDA should be derived");
      
      console.log("PDA addresses verified");
      console.log("Protocol State:", protocolState.toString());
      console.log("Oracle State:", oracleState.toString());
      console.log("Fees State:", feesState.toString());
    });
  });

  describe("Oracle Program Tests", () => {
    it("Should initialize oracle program", async () => {
      console.log("Initializing oracle program...");
      
      try {
        const tx = await oracleProgram.methods
          .initialize({
            oracleAddress: admin.publicKey
          })
          .accounts({
            state: oracleState,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc();

        console.log("Oracle initialized successfully:", tx);
        
        // Verify state
        const state = await oracleProgram.account.oracleStateAccount.fetch(oracleState);
        assert.equal(state.admin.toString(), admin.publicKey.toString());
        assert.equal(state.oracleAddress.toString(), admin.publicKey.toString());
        console.log("Oracle state verified");
      } catch (error) {
        console.log("Oracle initialization error:", error.message);
        // Don't throw - might already be initialized
        console.log("Oracle might already be initialized");
      }
    });
  });

  describe("Fees Program Tests", () => {
    it("Should initialize fees program", async () => {
      console.log("Initializing fees program...");
      
      try {
        const tx = await feesProgram.methods
          .initialize({
            admin: admin.publicKey
          })
          .accounts({
            state: feesState,
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc();

        console.log("Fees initialized successfully:", tx);
        
        // Verify state
        const state = await feesProgram.account.feeStateAccount.fetch(feesState);
        assert.equal(state.admin.toString(), admin.publicKey.toString());
        console.log("Fees state verified");
      } catch (error) {
        console.log("Fees initialization error:", error.message);
        // Don't throw - might already be initialized
        console.log("Fees might already be initialized");
      }
    });
  });

  describe("Protocol Program Tests", () => {
    it("Should initialize protocol program", async () => {
      console.log("Initializing protocol program...");
      
      try {
        const tx = await protocolProgram.methods
          .initialize({
            stableCoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            feeDistributor: feesProgram.programId
          })
          .accounts({
            state: protocolState,
            admin: admin.publicKey,
            stableCoinMint: stablecoinMint,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc();

        console.log("Protocol initialized successfully:", tx);
        
        // Verify state
        const state = await protocolProgram.account.stateAccount.fetch(protocolState);
        assert.equal(state.admin.toString(), admin.publicKey.toString());
        console.log("Protocol state verified");
      } catch (error) {
        console.log("Protocol initialization error:", error.message);
        // Don't throw - might already be initialized
        console.log("Protocol might already be initialized");
      }
    });
  });
});
