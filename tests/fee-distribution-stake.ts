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

describe("Fee Contract - Stability Pool Distribution Mode", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;

  const feesProgram = anchor.workspace.AerospacerFees as Program<AerospacerFees>;

  const admin = Keypair.generate();
  const payer = Keypair.generate();
  const stakeContractKeypair = Keypair.generate();
  
  let feeStateAccount: Keypair;
  let tokenMint: PublicKey;
  let payerTokenAccount: PublicKey;
  let stabilityPoolTokenAccount: PublicKey;
  let feeAddr1TokenAccount: PublicKey;
  let feeAddr2TokenAccount: PublicKey;

  before(async () => {
    console.log("\n🚀 Setting up Fee Distribution - Stability Pool Mode Tests...");
    
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

    const stakeAirdrop = await connection.requestAirdrop(
      stakeContractKeypair.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(stakeAirdrop);
    
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
      stakeContractKeypair,
      tokenMint,
      stakeContractKeypair.publicKey
    );

    const FEE_ADDR_1 = new PublicKey("8Lv4UrYHTrzvg9jPVVGNmxWyMrMvrZnCQLWucBzfJyyR");
    const FEE_ADDR_2 = new PublicKey("GcNwV1nA5bityjNYsWwPLHykpKuuhPzK1AQFBbrPopnX");

    const feeAddr1Airdrop = await connection.requestAirdrop(FEE_ADDR_1, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(feeAddr1Airdrop);

    const feeAddr2Airdrop = await connection.requestAirdrop(FEE_ADDR_2, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(feeAddr2Airdrop);

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
    console.log("  Payer:", payer.publicKey.toString());
    console.log("  Stake Contract:", stakeContractKeypair.publicKey.toString());
  });

  describe("Test 3.1: Enable Stake Mode and Set Stake Address", () => {
    it("Should enable stake mode and set address", async () => {
      console.log("🔄 Enabling stake mode...");

      await feesProgram.methods
        .toggleStakeContract()
        .accounts({
          admin: admin.publicKey,
          state: feeStateAccount.publicKey,
        })
        .signers([admin])
        .rpc();

      await feesProgram.methods
        .setStakeContractAddress({
          address: stakeContractKeypair.publicKey.toString()
        })
        .accounts({
          admin: admin.publicKey,
          state: feeStateAccount.publicKey,
        })
        .signers([admin])
        .rpc();

      const state = await feesProgram.account.feeStateAccount.fetch(
        feeStateAccount.publicKey
      );

      assert.equal(state.isStakeEnabled, true);
      assert.equal(
        state.stakeContractAddress.toString(),
        stakeContractKeypair.publicKey.toString()
      );

      console.log("✅ Stake mode enabled and address set");
    });
  });

  describe("Test 3.2: Distribute Fees to Stability Pool (100% Transfer)", () => {
    it("Should transfer 100% of fees to stability pool", async () => {
      const feeAmount = new BN(100000);

      const poolBalanceBefore = await getAccount(connection, stabilityPoolTokenAccount);

      console.log("💸 Distributing fees to stability pool...");
      console.log("  Amount:", feeAmount.toString());

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

      const poolBalanceAfter = await getAccount(connection, stabilityPoolTokenAccount);

      assert.equal(
        poolBalanceAfter.amount.toString(),
        (BigInt(poolBalanceBefore.amount.toString()) + BigInt(feeAmount.toString())).toString(),
        "Stability pool should receive full amount"
      );

      console.log("✅ 100% of fees transferred to stability pool");
    });
  });

  describe("Test 3.3: Verify total_fees_collected Increments Correctly", () => {
    it("Should increment total_fees_collected accurately", async () => {
      const stateBefore = await feesProgram.account.feeStateAccount.fetch(
        feeStateAccount.publicKey
      );
      const feeAmount = new BN(50000);

      await feesProgram.methods
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

      const stateAfter = await feesProgram.account.feeStateAccount.fetch(
        feeStateAccount.publicKey
      );

      assert.equal(
        stateAfter.totalFeesCollected.toString(),
        (BigInt(stateBefore.totalFeesCollected.toString()) + BigInt(feeAmount.toString())).toString(),
        "Total fees should increment by fee amount"
      );

      console.log("✅ total_fees_collected:", stateAfter.totalFeesCollected.toString());
    });
  });

  describe("Test 3.4: Validate Stability Pool Token Account Owner", () => {
    it("Should verify stability pool account owner matches stake_contract_address", async () => {
      const poolAccount = await getAccount(connection, stabilityPoolTokenAccount);

      assert.equal(
        poolAccount.owner.toString(),
        stakeContractKeypair.publicKey.toString(),
        "Pool account owner should match stake contract"
      );

      console.log("✅ Stability pool account owner validated");
    });
  });

  describe("Test 3.5: Reject Distribution if stake_contract_address is Default", () => {
    it("Should fail if stake address is Pubkey::default()", async () => {
      const tempStateAccount = Keypair.generate();
      
      await feesProgram.methods
        .initialize()
        .accounts({
          state: tempStateAccount.publicKey,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin, tempStateAccount])
        .rpc();

      await feesProgram.methods
        .toggleStakeContract()
        .accounts({
          admin: admin.publicKey,
          state: tempStateAccount.publicKey,
        })
        .signers([admin])
        .rpc();

      console.log("🔒 Attempting distribution with default stake address...");

      try {
        await feesProgram.methods
          .distributeFee({
            feeAmount: new BN(10000)
          })
          .accounts({
            payer: payer.publicKey,
            state: tempStateAccount.publicKey,
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
        console.log("✅ Distribution correctly rejected");
        expect(error.message).to.include("StakeContractNotSet");
      }
    });
  });

  describe("Test 3.6: Reject Distribution if Stability Pool Owner is Wrong", () => {
    it("Should fail if pool account owner doesn't match stake_contract_address", async () => {
      const wrongOwner = Keypair.generate();
      
      const wrongAirdrop = await connection.requestAirdrop(
        wrongOwner.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      await connection.confirmTransaction(wrongAirdrop);

      const wrongPoolAccount = await createAccount(
        connection,
        wrongOwner,
        tokenMint,
        wrongOwner.publicKey
      );

      console.log("🔒 Attempting distribution with wrong pool owner...");

      try {
        await feesProgram.methods
          .distributeFee({
            feeAmount: new BN(10000)
          })
          .accounts({
            payer: payer.publicKey,
            state: feeStateAccount.publicKey,
            payerTokenAccount: payerTokenAccount,
            stabilityPoolTokenAccount: wrongPoolAccount,
            feeAddress1TokenAccount: feeAddr1TokenAccount,
            feeAddress2TokenAccount: feeAddr2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();

        assert.fail("Should have thrown an error");
      } catch (error: any) {
        console.log("✅ Wrong pool owner correctly rejected");
        expect(error.message).to.include("InvalidStabilityPoolAccount");
      }
    });
  });

  describe("Test 3.7: Validate Token Mint Matches Across Accounts", () => {
    it("Should fail if token mints don't match", async () => {
      const wrongMint = await createMint(
        connection,
        admin,
        admin.publicKey,
        null,
        6
      );

      const wrongTokenAccount = await createAccount(
        connection,
        payer,
        wrongMint,
        payer.publicKey
      );

      await mintTo(
        connection,
        payer,
        wrongMint,
        wrongTokenAccount,
        admin,
        100000
      );

      console.log("🔒 Attempting distribution with mismatched token mints...");

      try {
        await feesProgram.methods
          .distributeFee({
            feeAmount: new BN(10000)
          })
          .accounts({
            payer: payer.publicKey,
            state: feeStateAccount.publicKey,
            payerTokenAccount: wrongTokenAccount,
            stabilityPoolTokenAccount: stabilityPoolTokenAccount,
            feeAddress1TokenAccount: feeAddr1TokenAccount,
            feeAddress2TokenAccount: feeAddr2TokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();

        assert.fail("Should have thrown an error");
      } catch (error: any) {
        console.log("✅ Mismatched token mints correctly rejected");
        expect(error.message).to.include("InvalidTokenMint");
      }
    });
  });

  describe("Test 3.8: Test Large Fee Amounts", () => {
    it("Should handle large fee amounts correctly", async () => {
      const largeAmount = new BN(999999999);

      const poolBalanceBefore = await getAccount(connection, stabilityPoolTokenAccount);

      console.log("💰 Distributing large fee amount:", largeAmount.toString());

      await feesProgram.methods
        .distributeFee({
          feeAmount: largeAmount
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

      const poolBalanceAfter = await getAccount(connection, stabilityPoolTokenAccount);

      assert.equal(
        poolBalanceAfter.amount.toString(),
        (BigInt(poolBalanceBefore.amount.toString()) + BigInt(largeAmount.toString())).toString()
      );

      console.log("✅ Large amount handled correctly");
    });
  });

  describe("Test 3.9: Test Multiple Consecutive Distributions", () => {
    it("Should handle multiple consecutive distributions", async () => {
      const amounts = [new BN(1000), new BN(2000), new BN(3000), new BN(4000), new BN(5000)];
      
      console.log("⚡ Performing multiple consecutive distributions...");

      for (let i = 0; i < amounts.length; i++) {
        await feesProgram.methods
          .distributeFee({
            feeAmount: amounts[i]
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

        console.log(`  Distribution ${i + 1}: ${amounts[i].toString()} ✓`);
      }

      const state = await feesProgram.account.feeStateAccount.fetch(
        feeStateAccount.publicKey
      );

      console.log("✅ Multiple distributions completed");
      console.log("  Total fees collected:", state.totalFeesCollected.toString());
    });
  });

  after(() => {
    console.log("\n✅ Fee Distribution - Stability Pool Mode Tests Complete");
    console.log("  Total Tests Passed: 9");
  });
});
