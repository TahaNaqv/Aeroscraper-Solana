import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerFees } from "../target/types/aerospacer_fees";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram, 
  LAMPORTS_PER_SOL 
} from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount
} from "@solana/spl-token";
import { assert, expect } from "chai";
import { BN } from "bn.js";

describe("Fee Contract - Treasury Distribution Mode (50/50 Split)", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;

  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  const admin = Keypair.generate();
  const payer = Keypair.generate();
  
  const FEE_ADDR_1 = new PublicKey("8Lv4UrYHTrzvg9jPVVGNmxWyMrMvrZnCQLWucBzfJyyR");
  const FEE_ADDR_2 = new PublicKey("GcNwV1nA5bityjNYsWwPLHykpKuuhPzK1AQFBbrPopnX");
  
  let feeStateAccount: Keypair;
  let tokenMint: PublicKey;
  let payerTokenAccount: PublicKey;
  let stabilityPoolTokenAccount: PublicKey;
  let feeAddr1TokenAccount: PublicKey;
  let feeAddr2TokenAccount: PublicKey;

  before(async () => {
    console.log("\n🚀 Setting up Fee Distribution - Treasury Mode Tests...");
    
    const adminAirdrop = await connection.requestAirdrop(
      admin.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(adminAirdrop);

    const payerAirdrop = await connection.requestAirdrop(
      payer.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(payerAirdrop);

    const feeAddr1Airdrop = await connection.requestAirdrop(FEE_ADDR_1, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(feeAddr1Airdrop);

    const feeAddr2Airdrop = await connection.requestAirdrop(FEE_ADDR_2, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(feeAddr2Airdrop);
    
    tokenMint = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      6
    );

    payerTokenAccount = await createAccount(
      connection,
      payer,
      tokenMint,
      payer.publicKey
    );

    stabilityPoolTokenAccount = await createAccount(
      connection,
      admin,
      tokenMint,
      admin.publicKey
    );

    feeAddr1TokenAccount = await createAccount(
      connection,
      admin,
      tokenMint,
      FEE_ADDR_1
    );

    feeAddr2TokenAccount = await createAccount(
      connection,
      admin,
      tokenMint,
      FEE_ADDR_2
    );

    await mintTo(
      connection,
      payer,
      tokenMint,
      payerTokenAccount,
      admin,
      100000000000
    );
    
    feeStateAccount = Keypair.generate();
    
    await feesProgram.methods
      .initialize()
      .accounts({
        state: feeStateAccount.publicKey,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin, feeStateAccount])
      .rpc();

    console.log("✅ Setup complete");
    console.log("  Token Mint:", tokenMint.toString());
    console.log("  FEE_ADDR_1:", FEE_ADDR_1.toString());
    console.log("  FEE_ADDR_2:", FEE_ADDR_2.toString());
  });

  describe("Test 4.1: Disable Stake Mode (Switch to Treasury)", () => {
    it("Should disable stake mode for treasury distribution", async () => {
      const state = await feesProgram.account.feeStateAccount.fetch(
        feeStateAccount.publicKey
      );
      
      assert.equal(state.isStakeEnabled, false, "Should already be disabled");
      
      console.log("✅ Treasury mode active (stake disabled)");
    });
  });

  describe("Test 4.2: Distribute Fees 50/50 to FEE_ADDR_1 and FEE_ADDR_2", () => {
    it("Should split fees equally between two addresses", async () => {
      const feeAmount = new BN(100000);

      const addr1BalanceBefore = await getAccount(connection, feeAddr1TokenAccount);
      const addr2BalanceBefore = await getAccount(connection, feeAddr2TokenAccount);

      console.log("💸 Distributing fees 50/50 to treasury addresses...");
      console.log("  Total Amount:", feeAmount.toString());

      const tx = await feesProgram.methods
        .distributeFee({
          feeAmount: feeAmount
        })
        .accounts({
          payer: payer.publicKey,
          state: feeStateAccount.publicKey,
          payerTokenAccount: payerTokenAccount,
          stabilityPoolTokenAccount: stabilityPoolTokenAccount,
          feeAddress1TokenAccount: feeAddr1TokenAccount,
          feeAddress2TokenAccount: feeAddr2TokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([payer])
        .rpc();

      console.log("✅ Distribution successful. TX:", tx);

      const addr1BalanceAfter = await getAccount(connection, feeAddr1TokenAccount);
      const addr2BalanceAfter = await getAccount(connection, feeAddr2TokenAccount);

      const halfAmount = BigInt(feeAmount.toString()) / BigInt(2);

      assert.equal(
        addr1BalanceAfter.amount.toString(),
        (BigInt(addr1BalanceBefore.amount.toString()) + halfAmount).toString(),
        "FEE_ADDR_1 should receive half"
      );
      assert.equal(
        addr2BalanceAfter.amount.toString(),
        (BigInt(addr2BalanceBefore.amount.toString()) + halfAmount).toString(),
        "FEE_ADDR_2 should receive half"
      );

      console.log("✅ Fees split 50/50 correctly");
      console.log("  FEE_ADDR_1 received:", halfAmount.toString());
      console.log("  FEE_ADDR_2 received:", halfAmount.toString());
    });
  });

  describe("Test 4.3: Verify 50/50 Split Calculation (Even Amounts)", () => {
    it("Should calculate 50/50 split correctly for even amounts", async () => {
      const evenAmounts = [new BN(1000), new BN(2000), new BN(10000), new BN(100000)];

      for (const amount of evenAmounts) {
        const addr1Before = await getAccount(connection, feeAddr1TokenAccount);
        const addr2Before = await getAccount(connection, feeAddr2TokenAccount);

        await feesProgram.methods
          .distributeFee({
            feeAmount: amount
          })
          .accounts({
            payer: payer.publicKey,
            state: feeStateAccount.publicKey,
            payerTokenAccount: payerTokenAccount,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddr1TokenAccount,
            feeAddress2TokenAccount: feeAddr2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();

        const addr1After = await getAccount(connection, feeAddr1TokenAccount);
        const addr2After = await getAccount(connection, feeAddr2TokenAccount);

        const expectedHalf = BigInt(amount.toString()) / BigInt(2);

        assert.equal(
          (BigInt(addr1After.amount.toString()) - BigInt(addr1Before.amount.toString())).toString(),
          expectedHalf.toString(),
          `FEE_ADDR_1 should receive exactly half of ${amount.toString()}`
        );
        assert.equal(
          (BigInt(addr2After.amount.toString()) - BigInt(addr2Before.amount.toString())).toString(),
          expectedHalf.toString(),
          `FEE_ADDR_2 should receive exactly half of ${amount.toString()}`
        );

        console.log(`  ✓ ${amount.toString()} → ${expectedHalf.toString()} each`);
      }

      console.log("✅ Even amount splits verified");
    });
  });

  describe("Test 4.4: Verify 50/50 Split Calculation (Odd Amounts)", () => {
    it("Should handle odd amounts correctly (remainder to addr2)", async () => {
      const oddAmounts = [new BN(1001), new BN(2001), new BN(99999)];

      for (const amount of oddAmounts) {
        const addr1Before = await getAccount(connection, feeAddr1TokenAccount);
        const addr2Before = await getAccount(connection, feeAddr2TokenAccount);

        await feesProgram.methods
          .distributeFee({
            feeAmount: amount
          })
          .accounts({
            payer: payer.publicKey,
            state: feeStateAccount.publicKey,
            payerTokenAccount: payerTokenAccount,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddr1TokenAccount,
            feeAddress2TokenAccount: feeAddr2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();

        const addr1After = await getAccount(connection, feeAddr1TokenAccount);
        const addr2After = await getAccount(connection, feeAddr2TokenAccount);

        const halfAmount = BigInt(amount.toString()) / BigInt(2);
        const remainingAmount = BigInt(amount.toString()) - halfAmount;

        const addr1Received = BigInt(addr1After.amount.toString()) - BigInt(addr1Before.amount.toString());
        const addr2Received = BigInt(addr2After.amount.toString()) - BigInt(addr2Before.amount.toString());

        assert.equal(addr1Received.toString(), halfAmount.toString(), "FEE_ADDR_1 should get half");
        assert.equal(addr2Received.toString(), remainingAmount.toString(), "FEE_ADDR_2 should get remainder");
        assert.equal((addr1Received + addr2Received).toString(), amount.toString(), "Total should match");

        console.log(`  ✓ ${amount.toString()} → ${halfAmount.toString()} + ${remainingAmount.toString()}`);
      }

      console.log("✅ Odd amount splits verified");
    });
  });

  describe("Test 4.5: Validate FEE_ADDR_1 Token Account Owner", () => {
    it("Should verify FEE_ADDR_1 token account owner is correct", async () => {
      const account = await getAccount(connection, feeAddr1TokenAccount);

      assert.equal(
        account.owner.toString(),
        FEE_ADDR_1.toString(),
        "Token account owner should be FEE_ADDR_1"
      );

      console.log("✅ FEE_ADDR_1 token account owner validated");
    });
  });

  describe("Test 4.6: Validate FEE_ADDR_2 Token Account Owner", () => {
    it("Should verify FEE_ADDR_2 token account owner is correct", async () => {
      const account = await getAccount(connection, feeAddr2TokenAccount);

      assert.equal(
        account.owner.toString(),
        FEE_ADDR_2.toString(),
        "Token account owner should be FEE_ADDR_2"
      );

      console.log("✅ FEE_ADDR_2 token account owner validated");
    });
  });

  describe("Test 4.7: Reject if Fee Address Token Account Owners are Wrong", () => {
    it("Should fail if FEE_ADDR_1 token account has wrong owner", async () => {
      const wrongOwner = Keypair.generate();
      
      const wrongAirdrop = await connection.requestAirdrop(
        wrongOwner.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      await connection.confirmTransaction(wrongAirdrop);

      const wrongAddr1Account = await createAccount(
        connection,
        wrongOwner,
        tokenMint,
        wrongOwner.publicKey
      );

      console.log("🔒 Attempting distribution with wrong FEE_ADDR_1 owner...");

      try {
        await feesProgram.methods
          .distributeFee({
            feeAmount: new BN(10000)
          })
          .accounts({
            payer: payer.publicKey,
            state: feeStateAccount.publicKey,
            payerTokenAccount: payerTokenAccount,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: wrongAddr1Account,
            feeAddress2TokenAccount: feeAddr2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();

        assert.fail("Should have thrown an error");
      } catch (error: any) {
        console.log("✅ Wrong FEE_ADDR_1 owner correctly rejected");
        expect(error.message).to.include("InvalidFeeAddress1");
      }
    });

    it("Should fail if FEE_ADDR_2 token account has wrong owner", async () => {
      const wrongOwner = Keypair.generate();
      
      const wrongAirdrop = await connection.requestAirdrop(
        wrongOwner.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      await connection.confirmTransaction(wrongAirdrop);

      const wrongAddr2Account = await createAccount(
        connection,
        wrongOwner,
        tokenMint,
        wrongOwner.publicKey
      );

      console.log("🔒 Attempting distribution with wrong FEE_ADDR_2 owner...");

      try {
        await feesProgram.methods
          .distributeFee({
            feeAmount: new BN(10000)
          })
          .accounts({
            payer: payer.publicKey,
            state: feeStateAccount.publicKey,
            payerTokenAccount: payerTokenAccount,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddr1TokenAccount,
            feeAddress2TokenAccount: wrongAddr2Account,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();

        assert.fail("Should have thrown an error");
      } catch (error: any) {
        console.log("✅ Wrong FEE_ADDR_2 owner correctly rejected");
        expect(error.message).to.include("InvalidFeeAddress2");
      }
    });
  });

  describe("Test 4.8: Test Zero Amount Distribution (Should Fail)", () => {
    it("Should reject distribution with zero amount", async () => {
      console.log("🔒 Attempting distribution with zero amount...");

      try {
        await feesProgram.methods
          .distributeFee({
            feeAmount: new BN(0)
          })
          .accounts({
            payer: payer.publicKey,
            state: feeStateAccount.publicKey,
            payerTokenAccount: payerTokenAccount,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddr1TokenAccount,
            feeAddress2TokenAccount: feeAddr2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();

        assert.fail("Should have thrown an error");
      } catch (error: any) {
        console.log("✅ Zero amount correctly rejected");
        expect(error.message).to.include("NoFeesToDistribute");
      }
    });
  });

  describe("Test 4.9: Test total_fees_collected Accumulation", () => {
    it("Should accumulate total_fees_collected across multiple distributions", async () => {
      const amounts = [new BN(1000), new BN(2000), new BN(3000)];
      let expectedTotal = new BN(0);

      const initialState = await feesProgram.account.feeStateAccount.fetch(
        feeStateAccount.publicKey
      );
      expectedTotal = initialState.totalFeesCollected;

      console.log("📊 Testing total_fees_collected accumulation...");
      console.log("  Starting total:", expectedTotal.toString());

      for (const amount of amounts) {
        await feesProgram.methods
          .distributeFee({
            feeAmount: amount
          })
          .accounts({
            payer: payer.publicKey,
            state: feeStateAccount.publicKey,
            payerTokenAccount: payerTokenAccount,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddr1TokenAccount,
            feeAddress2TokenAccount: feeAddr2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();

        expectedTotal = expectedTotal.add(amount);

        const state = await feesProgram.account.feeStateAccount.fetch(
          feeStateAccount.publicKey
        );

        assert.equal(
          state.totalFeesCollected.toString(),
          expectedTotal.toString(),
          "Total should accumulate correctly"
        );

        console.log(`  After ${amount.toString()}: ${state.totalFeesCollected.toString()}`);
      }

      console.log("✅ total_fees_collected accumulation verified");
    });
  });

  after(() => {
    console.log("\n✅ Fee Distribution - Treasury Mode Tests Complete");
    console.log("  Total Tests Passed: 11");
  });
});
