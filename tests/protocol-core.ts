import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import { assert } from "chai";

describe("Aeroscraper Protocol Core Operations", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const oracleProgram = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  // Test accounts
  const admin = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();

  // Token mints
  let stablecoinMint: PublicKey;
  let collateralMint: PublicKey;

  // Token accounts
  let adminStablecoinAccount: PublicKey;
  let adminCollateralAccount: PublicKey;
  let user1StablecoinAccount: PublicKey;
  let user1CollateralAccount: PublicKey;
  let user2StablecoinAccount: PublicKey;
  let user2CollateralAccount: PublicKey;

  // Program state accounts
  let protocolState: PublicKey;
  let oracleState: PublicKey;
  let feesState: PublicKey;

  // User trove accounts
  let user1Trove: PublicKey;
  let user2Trove: PublicKey;

  // User stake accounts
  let user1Stake: PublicKey;
  let user2Stake: PublicKey;

  before(async () => {
    // Airdrop SOL to admin
    const signature = await provider.connection.requestAirdrop(admin.publicKey, 1000000000);
    await provider.connection.confirmTransaction(signature);

    // Create token mints
    stablecoinMint = await createMint(provider.connection, admin, admin.publicKey, null, 6);
    collateralMint = await createMint(provider.connection, admin, admin.publicKey, null, 6);

    // Create token accounts
    adminStablecoinAccount = await getAssociatedTokenAddress(stablecoinMint, admin.publicKey);
    adminCollateralAccount = await getAssociatedTokenAddress(collateralMint, admin.publicKey);
    user1StablecoinAccount = await getAssociatedTokenAddress(stablecoinMint, user1.publicKey);
    user1CollateralAccount = await getAssociatedTokenAddress(collateralMint, user1.publicKey);
    user2StablecoinAccount = await getAssociatedTokenAddress(stablecoinMint, user2.publicKey);
    user2CollateralAccount = await getAssociatedTokenAddress(collateralMint, user2.publicKey);

    // Create token accounts
    await createAssociatedTokenAccount(provider.connection, admin, admin.publicKey, stablecoinMint);
    await createAssociatedTokenAccount(provider.connection, admin, admin.publicKey, collateralMint);
    await createAssociatedTokenAccount(provider.connection, admin, user1.publicKey, stablecoinMint);
    await createAssociatedTokenAccount(provider.connection, admin, user1.publicKey, collateralMint);
    await createAssociatedTokenAccount(provider.connection, admin, user2.publicKey, stablecoinMint);
    await createAssociatedTokenAccount(provider.connection, admin, user2.publicKey, collateralMint);

    // Mint initial tokens
    await mintTo(provider.connection, admin, stablecoinMint, adminStablecoinAccount, admin, 1000000000);
    await mintTo(provider.connection, admin, collateralMint, adminCollateralAccount, admin, 1000000000);

    // Transfer tokens to users
    await transfer(provider.connection, admin, adminCollateralAccount, user1CollateralAccount, admin, 100000000);
    await transfer(provider.connection, admin, adminCollateralAccount, user2CollateralAccount, admin, 100000000);

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

    // Derive user trove PDAs
    const [user1TrovePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("trove"), user1.publicKey.toBuffer()],
      protocolProgram.programId
    );
    const [user2TrovePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("trove"), user2.publicKey.toBuffer()],
      protocolProgram.programId
    );

    user1Trove = user1TrovePda;
    user2Trove = user2TrovePda;

    // Derive user stake PDAs
    const [user1StakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), user1.publicKey.toBuffer()],
      protocolProgram.programId
    );
    const [user2StakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), user2.publicKey.toBuffer()],
      protocolProgram.programId
    );

    user1Stake = user1StakePda;
    user2Stake = user2StakePda;

    // Initialize programs
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
    } catch (e) {
      console.log("Oracle already initialized");
    }

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
    } catch (e) {
      console.log("Fees already initialized");
    }

    try {
      await protocolProgram.methods
        .initialize()
        .accounts({
          state: protocolState,
          admin: admin.publicKey,
          oracleProgram: oracleProgram.programId,
          oracleState: oracleState,
          feesProgram: feesProgram.programId,
          feesState: feesState,
          stablecoinMint: stablecoinMint,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([admin])
        .rpc();
    } catch (e) {
      console.log("Protocol already initialized");
    }
  });

  describe("Core Protocol Operations", () => {
    it("Should open a trove successfully", async () => {
      const collateralAmount = 10000000; // 10 tokens
      const loanAmount = 5000000; // 5 stablecoins
      const collateralDenom = "SOL";

      try {
        await protocolProgram.methods
          .openTrove({
            collateralAmount: new anchor.BN(collateralAmount),
            loanAmount: new anchor.BN(loanAmount),
            collateralDenom: collateralDenom,
          })
          .accounts({
            trove: user1Trove,
            state: protocolState,
            user: user1.publicKey,
            userCollateralAccount: user1CollateralAccount,
            userStablecoinAccount: user1StablecoinAccount,
            stablecoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Trove opened successfully");
      } catch (error) {
        console.log("❌ Trove opening failed:", error);
        throw error;
      }
    });

    it("Should add collateral to existing trove", async () => {
      const additionalCollateral = 5000000; // 5 more tokens

      try {
        await protocolProgram.methods
          .addCollateral({
            collateralAmount: new anchor.BN(additionalCollateral),
            collateralDenom: "SOL",
          })
          .accounts({
            trove: user1Trove,
            state: protocolState,
            user: user1.publicKey,
            userCollateralAccount: user1CollateralAccount,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Collateral added successfully");
      } catch (error) {
        console.log("❌ Adding collateral failed:", error);
        throw error;
      }
    });

    it("Should borrow additional loan from trove", async () => {
      const additionalLoan = 2000000; // 2 more stablecoins

      try {
        await protocolProgram.methods
          .borrowLoan({
            loanAmount: new anchor.BN(additionalLoan),
            collateralDenom: "SOL",
          })
          .accounts({
            trove: user1Trove,
            state: protocolState,
            user: user1.publicKey,
            userStablecoinAccount: user1StablecoinAccount,
            stablecoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Additional loan borrowed successfully");
      } catch (error) {
        console.log("❌ Borrowing additional loan failed:", error);
        throw error;
      }
    });

    it("Should stake stablecoins in stability pool", async () => {
      const stakeAmount = 3000000; // 3 stablecoins

      try {
        await protocolProgram.methods
          .stake({
            amount: new anchor.BN(stakeAmount),
          })
          .accounts({
            stake: user1Stake,
            state: protocolState,
            user: user1.publicKey,
            userStablecoinAccount: user1StablecoinAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Stablecoins staked successfully");
      } catch (error) {
        console.log("❌ Staking failed:", error);
        throw error;
      }
    });

    it("Should open a second trove for user2", async () => {
      const collateralAmount = 8000000; // 8 tokens
      const loanAmount = 3000000; // 3 stablecoins
      const collateralDenom = "SOL";

      try {
        await protocolProgram.methods
          .openTrove({
            collateralAmount: new anchor.BN(collateralAmount),
            loanAmount: new anchor.BN(loanAmount),
            collateralDenom: collateralDenom,
          })
          .accounts({
            trove: user2Trove,
            state: protocolState,
            user: user2.publicKey,
            userCollateralAccount: user2CollateralAccount,
            userStablecoinAccount: user2StablecoinAccount,
            stablecoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            feesProgram: feesProgram.programId,
            feesState: feesState,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([user2])
          .rpc();

        console.log("✅ Second trove opened successfully");
      } catch (error) {
        console.log("❌ Second trove opening failed:", error);
        throw error;
      }
    });

    it("Should repay loan partially", async () => {
      const repayAmount = 2000000; // 2 stablecoins

      try {
        await protocolProgram.methods
          .repayLoan({
            repayAmount: new anchor.BN(repayAmount),
            collateralDenom: "SOL",
          })
          .accounts({
            trove: user1Trove,
            state: protocolState,
            user: user1.publicKey,
            userStablecoinAccount: user1StablecoinAccount,
            stablecoinMint: stablecoinMint,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Loan partially repaid successfully");
      } catch (error) {
        console.log("❌ Loan repayment failed:", error);
        throw error;
      }
    });

    it("Should remove collateral from trove", async () => {
      const removeAmount = 3000000; // 3 tokens

      try {
        await protocolProgram.methods
          .removeCollateral({
            collateralAmount: new anchor.BN(removeAmount),
            collateralDenom: "SOL",
          })
          .accounts({
            trove: user1Trove,
            state: protocolState,
            user: user1.publicKey,
            userCollateralAccount: user1CollateralAccount,
            oracleProgram: oracleProgram.programId,
            oracleState: oracleState,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Collateral removed successfully");
      } catch (error) {
        console.log("❌ Removing collateral failed:", error);
        throw error;
      }
    });

    it("Should unstake stablecoins from stability pool", async () => {
      const unstakeAmount = 1500000; // 1.5 stablecoins

      try {
        await protocolProgram.methods
          .unstake({
            amount: new anchor.BN(unstakeAmount),
          })
          .accounts({
            stake: user1Stake,
            state: protocolState,
            user: user1.publicKey,
            userStablecoinAccount: user1StablecoinAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        console.log("✅ Stablecoins unstaked successfully");
      } catch (error) {
        console.log("❌ Unstaking failed:", error);
        throw error;
      }
    });
  });

  describe("Protocol State Verification", () => {
    it("Should verify protocol state after operations", async () => {
      try {
        const stateAccount = await protocolProgram.account.stateAccount.fetch(protocolState);
        console.log("📊 Protocol State:");
        console.log("- Total Debt Amount:", stateAccount.totalDebtAmount.toString());
        console.log("- Total Collateral Amount:", stateAccount.totalCollateralAmount.toString());
        console.log("- Total Stake Amount:", stateAccount.totalStakeAmount.toString());
        console.log("- Minimum Collateral Ratio:", stateAccount.minimumCollateralRatio.toString());
        console.log("- Protocol Fee:", stateAccount.protocolFee.toString());
        
        assert(stateAccount.totalDebtAmount.gt(new anchor.BN(0)), "Total debt should be greater than 0");
        assert(stateAccount.totalCollateralAmount.gt(new anchor.BN(0)), "Total collateral should be greater than 0");
        assert(stateAccount.totalStakeAmount.gt(new anchor.BN(0)), "Total stake should be greater than 0");
        
        console.log("✅ Protocol state verification passed");
      } catch (error) {
        console.log("❌ Protocol state verification failed:", error);
        throw error;
      }
    });

    it("Should verify user trove state", async () => {
      try {
        const troveAccount = await protocolProgram.account.troveAccount.fetch(user1Trove);
        console.log("📊 User1 Trove State:");
        console.log("- Owner:", troveAccount.owner.toString());
        console.log("- Debt Amount:", troveAccount.debtAmount.toString());
        console.log("- Collateral Amount:", troveAccount.collateralAmount.toString());
        console.log("- Collateral Ratio:", troveAccount.collateralRatio.toString());
        console.log("- Is Active:", troveAccount.isActive);
        
        assert(troveAccount.owner.equals(user1.publicKey), "Trove owner should match user1");
        assert(troveAccount.isActive, "Trove should be active");
        assert(troveAccount.debtAmount.gt(new anchor.BN(0)), "Debt amount should be greater than 0");
        assert(troveAccount.collateralAmount.gt(new anchor.BN(0)), "Collateral amount should be greater than 0");
        
        console.log("✅ User trove verification passed");
      } catch (error) {
        console.log("❌ User trove verification failed:", error);
        throw error;
      }
    });

    it("Should verify user stake state", async () => {
      try {
        const stakeAccount = await protocolProgram.account.stakeAccount.fetch(user1Stake);
        console.log("📊 User1 Stake State:");
        console.log("- Owner:", stakeAccount.owner.toString());
        console.log("- Amount:", stakeAccount.amount.toString());
        console.log("- Total Stake At Time:", stakeAccount.totalStakeAtTime.toString());
        console.log("- Percentage:", stakeAccount.percentage.toString());
        
        assert(stakeAccount.owner.equals(user1.publicKey), "Stake owner should match user1");
        assert(stakeAccount.amount.gt(new anchor.BN(0)), "Stake amount should be greater than 0");
        
        console.log("✅ User stake verification passed");
      } catch (error) {
        console.log("❌ User stake verification failed:", error);
        throw error;
      }
    });
  });
});

// Helper functions
import { 
  createMint as splCreateMint,
  createAssociatedTokenAccount as splCreateAssociatedTokenAccount,
  mintTo as splMintTo,
  transfer as splTransfer,
  getMinimumBalanceForRentExemption
} from "@solana/spl-token";

async function createMint(
  connection: anchor.web3.Connection,
  payer: Keypair,
  mintAuthority: PublicKey,
  freezeAuthority: PublicKey | null,
  decimals: number
): Promise<PublicKey> {
  return await splCreateMint(
    connection,
    payer,
    mintAuthority,
    freezeAuthority,
    decimals,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID
  );
}

async function createAssociatedTokenAccount(
  connection: anchor.web3.Connection,
  payer: Keypair,
  owner: PublicKey,
  mint: PublicKey
): Promise<void> {
  await splCreateAssociatedTokenAccount(
    connection,
    payer,
    owner,
    mint,
    undefined,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
}

async function mintTo(
  connection: anchor.web3.Connection,
  payer: Keypair,
  mint: PublicKey,
  destination: PublicKey,
  authority: Keypair,
  amount: number
): Promise<void> {
  await splMintTo(
    connection,
    payer,
    mint,
    destination,
    authority,
    amount,
    [],
    undefined,
    TOKEN_PROGRAM_ID
  );
}

async function transfer(
  connection: anchor.web3.Connection,
  payer: Keypair,
  source: PublicKey,
  destination: PublicKey,
  owner: Keypair,
  amount: number
): Promise<void> {
  await splTransfer(
    connection,
    payer,
    source,
    destination,
    owner,
    amount,
    [],
    undefined,
    TOKEN_PROGRAM_ID
  );
}
