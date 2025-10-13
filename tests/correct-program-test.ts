import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { assert } from "chai";

describe("Correct Program IDs Test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Use the actual deployed program IDs from solana program show
  const PROTOCOL_PROGRAM_ID = "6qjCWo3diZmBwTHVn1CfegbNQxBMy1q2MqPHUEN5xja2";
  const ORACLE_PROGRAM_ID = "D8xkMuN8J1v7kH6R8Xd4RwMcTk1HETgfFN24sSB3ZoFJ";
  const FEES_PROGRAM_ID = "h4ka5hAgZ5Ez7x4bjMiAqQHnuwnfry3aBiWNzUw3F7t";

  // Test accounts
  const admin = Keypair.generate();

  // Program state accounts
  let protocolState: PublicKey;
  let oracleState: PublicKey;
  let feesState: PublicKey;

  before(async () => {
    // Airdrop SOL to admin
    const signature = await provider.connection.requestAirdrop(admin.publicKey, 1000000000);
    await provider.connection.confirmTransaction(signature);

    // Derive state PDAs using the correct program IDs
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

    console.log("✅ Setup completed with correct program IDs");
    console.log("- Protocol Program ID:", PROTOCOL_PROGRAM_ID);
    console.log("- Oracle Program ID:", ORACLE_PROGRAM_ID);
    console.log("- Fees Program ID:", FEES_PROGRAM_ID);
    console.log("- Protocol State:", protocolState.toString());
    console.log("- Oracle State:", oracleState.toString());
    console.log("- Fees State:", feesState.toString());
  });

  describe("Program Accessibility", () => {
    it("Should verify protocol program is accessible", async () => {
      try {
        const programInfo = await provider.connection.getAccountInfo(new PublicKey(PROTOCOL_PROGRAM_ID));
        assert(programInfo !== null, "Protocol program should exist");
        console.log("✅ Protocol program is accessible");
        console.log("- Program size:", programInfo?.data.length, "bytes");
        console.log("- Program owner:", programInfo?.owner.toString());
      } catch (error) {
        console.log("❌ Protocol program accessibility failed:", error);
        throw error;
      }
    });

    it("Should verify oracle program is accessible", async () => {
      try {
        const programInfo = await provider.connection.getAccountInfo(new PublicKey(ORACLE_PROGRAM_ID));
        assert(programInfo !== null, "Oracle program should exist");
        console.log("✅ Oracle program is accessible");
        console.log("- Program size:", programInfo?.data.length, "bytes");
        console.log("- Program owner:", programInfo?.owner.toString());
      } catch (error) {
        console.log("❌ Oracle program accessibility failed:", error);
        throw error;
      }
    });

    it("Should verify fees program is accessible", async () => {
      try {
        const programInfo = await provider.connection.getAccountInfo(new PublicKey(FEES_PROGRAM_ID));
        assert(programInfo !== null, "Fees program should exist");
        console.log("✅ Fees program is accessible");
        console.log("- Program size:", programInfo?.data.length, "bytes");
        console.log("- Program owner:", programInfo?.owner.toString());
      } catch (error) {
        console.log("❌ Fees program accessibility failed:", error);
        throw error;
      }
    });
  });

  describe("State Account Verification", () => {
    it("Should check if protocol state account exists", async () => {
      try {
        const accountInfo = await provider.connection.getAccountInfo(protocolState);
        if (accountInfo === null) {
          console.log("ℹ️ Protocol state account does not exist yet (expected for new deployment)");
        } else {
          console.log("✅ Protocol state account exists");
          console.log("- Account size:", accountInfo.data.length, "bytes");
          console.log("- Account owner:", accountInfo.owner.toString());
        }
      } catch (error) {
        console.log("❌ Protocol state account check failed:", error);
        throw error;
      }
    });

    it("Should check if oracle state account exists", async () => {
      try {
        const accountInfo = await provider.connection.getAccountInfo(oracleState);
        if (accountInfo === null) {
          console.log("ℹ️ Oracle state account does not exist yet (expected for new deployment)");
        } else {
          console.log("✅ Oracle state account exists");
          console.log("- Account size:", accountInfo.data.length, "bytes");
          console.log("- Account owner:", accountInfo.owner.toString());
        }
      } catch (error) {
        console.log("❌ Oracle state account check failed:", error);
        throw error;
      }
    });

    it("Should check if fees state account exists", async () => {
      try {
        const accountInfo = await provider.connection.getAccountInfo(feesState);
        if (accountInfo === null) {
          console.log("ℹ️ Fees state account does not exist yet (expected for new deployment)");
        } else {
          console.log("✅ Fees state account exists");
          console.log("- Account size:", accountInfo.data.length, "bytes");
          console.log("- Account owner:", accountInfo.owner.toString());
        }
      } catch (error) {
        console.log("❌ Fees state account check failed:", error);
        throw error;
      }
    });
  });

  describe("Program Deployment Status", () => {
    it("Should verify all programs are deployed and accessible", async () => {
      const programs = [
        { name: "Protocol", id: PROTOCOL_PROGRAM_ID },
        { name: "Oracle", id: ORACLE_PROGRAM_ID },
        { name: "Fees", id: FEES_PROGRAM_ID }
      ];

      for (const program of programs) {
        try {
          const programInfo = await provider.connection.getAccountInfo(new PublicKey(program.id));
          assert(programInfo !== null, `${program.name} program should exist`);
          console.log(`✅ ${program.name} program is deployed and accessible`);
        } catch (error) {
          console.log(`❌ ${program.name} program check failed:`, error);
          throw error;
        }
      }
    });
  });
});
