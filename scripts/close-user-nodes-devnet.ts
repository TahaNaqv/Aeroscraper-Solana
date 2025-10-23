import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { PublicKey } from "@solana/web3.js";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const admin = provider.wallet as anchor.Wallet;

  console.log("🧹 Closing corrupted user Node accounts on devnet...");
  console.log("Admin:", admin.publicKey.toString());
  console.log("Protocol:", protocolProgram.programId.toString());
  console.log();

  const [statePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("state")],
    protocolProgram.programId
  );

  console.log("State PDA:", statePDA.toString());
  console.log();

  const userPublicKeys = [
    "GLThBB9YBHJgQMVj2wGpbLwKwEMfjr1ieVHaqgueYxUn",
    "H6jcCMEZJvBsyd16VizDF9E6aosUpzuTvjuhk1JWWavw",
  ];

  for (const userPubkeyStr of userPublicKeys) {
    try {
      const userPubkey = new PublicKey(userPubkeyStr);
      
      const [nodePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("node"), userPubkey.toBuffer()],
        protocolProgram.programId
      );

      console.log(`Checking Node for user ${userPubkeyStr}...`);
      console.log(`  Node PDA: ${nodePDA.toString()}`);

      const nodeAccountInfo = await provider.connection.getAccountInfo(nodePDA);
      
      if (!nodeAccountInfo) {
        console.log(`  ✓ Node doesn't exist (already clean)`);
        console.log();
        continue;
      }

      console.log(`  ⚠️  Node exists with ${nodeAccountInfo.data.length} bytes`);

      try {
        const nodeData = await protocolProgram.account.node.fetch(nodePDA);
        console.log(`  Current Node data:`);
        console.log(`    ID: ${nodeData.id.toString()}`);
        console.log(`    Prev: ${nodeData.prevId?.toString() || 'null'}`);
        console.log(`    Next: ${nodeData.nextId?.toString() || 'null'}`);
      } catch (e) {
        console.log(`  ⚠️  Failed to decode Node (corrupted): ${e.message}`);
      }

      console.log(`  📤 Calling close_node instruction...`);

      const tx = await protocolProgram.methods
        .closeNode(userPubkey)
        .accounts({
          node: nodePDA,
          state: statePDA,
          authority: admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .rpc();

      console.log(`  ✅ Success! Transaction: ${tx}`);
      console.log();
    } catch (error) {
      console.error(`  ❌ Error closing Node for ${userPubkeyStr}:`, error.message);
      console.log();
    }
  }

  console.log("✅ Node cleanup complete!");
  console.log();
  console.log("💡 You can now run your tests:");
  console.log("   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \\");
  console.log("   ANCHOR_WALLET=~/.config/solana/id.json \\");
  console.log("   npx ts-mocha -p ./tsconfig.json -t 1000000 'tests/**/protocol-core.ts'");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
